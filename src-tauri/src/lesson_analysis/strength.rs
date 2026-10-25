use super::*;

pub(super) fn build_strength(
    facts: &LessonFacts,
    interactions: &[QuantModelInteraction],
) -> LessonStrengthAnalysis {
    let day_element = Element::from_name(&facts.day_master.element);
    let month_element = Element::from_name(&facts.month_command.element);
    let same_season_element = day_element == month_element;
    let seal_in_season = month_element == day_element.get_reinforced();
    let support_stem_count = facts
        .pillars
        .iter()
        .enumerate()
        .filter(|(index, pillar)| {
            *index != 2
                && matches!(
                    family_for_element(&day_element, &Element::from_name(&pillar.stem_element)),
                    "印" | "比劫"
                )
        })
        .count();
    let support_branch_count = facts
        .pillars
        .iter()
        .filter(|pillar| {
            matches!(
                family_for_element(
                    &day_element,
                    &Element::from_name(&pillar.branch_main_element)
                ),
                "印" | "比劫"
            )
        })
        .count();
    // 得勢 (2026-09-02, 2026-09-09): count all eight characters (stems and branch main qi, day
    // master included) as 同黨 (印、比劫) or 異黨 (食傷、財、官殺). The larger party wins; on a tie
    // 「得令者為旺」: the party holding the month command wins.
    let is_same_party = |element: &str| {
        matches!(
            family_for_element(&day_element, &Element::from_name(element)),
            "印" | "比劫"
        )
    };
    let same_party_count = facts
        .pillars
        .iter()
        .flat_map(|pillar| [pillar.stem_element.as_str(), pillar.branch_main_element.as_str()])
        .filter(|element| is_same_party(element))
        .count();
    let opposing_party_count = facts.pillars.len() * 2 - same_party_count;
    let month_command_same_party = is_same_party(&facts.month_command.element);
    let party_tie = same_party_count == opposing_party_count;
    let momentum_proxy =
        same_party_count > opposing_party_count || (party_tie && month_command_same_party);
    let rooted = !facts.roots.is_empty();
    let observation_count =
        usize::from(same_season_element) + usize::from(momentum_proxy) + usize::from(rooted);
    let pressure_families_surface = pressure_families(facts, true);
    let pressure_families_hidden = pressure_families(facts, false);
    // On a 四比四 tie the teacher reads the month-command party as decisive (2026-09-02 source
    // chart: 金得令 → 身強), even when 得時 and 得地 are both missing.
    let tie_won_by_month = party_tie && month_command_same_party;
    let classification = match observation_count {
        3 if pressure_families_surface.is_empty() => "旺極候選",
        3 => "旺候選",
        2 => "強候選",
        1 if tie_won_by_month => "強候選",
        1 => "弱候選",
        _ if PRESSURE_FAMILIES
            .iter()
            .all(|family| pressure_families_surface.iter().any(|item| item == family)) =>
        {
            "衰極候選"
        }
        _ => "依據不足",
    }
    .to_string();
    let checks = strength_checks(
        facts,
        StrengthCheckInputs {
            same_season_element,
            seal_in_season,
            same_party_count,
            opposing_party_count,
            month_command_same_party,
            momentum_proxy,
        },
    );
    let (same_party_characters, opposing_party_characters) = party_characters(facts, &is_same_party);
    let verdict_reason = verdict_reason(
        &checks,
        &classification,
        &pressure_families_surface,
        tie_won_by_month,
    );
    let mut ordered_interactions = interactions.iter().collect::<Vec<_>>();
    ordered_interactions.sort_by(|left, right| {
        left.scope
            .cmp(&right.scope)
            .then_with(|| left.r#type.cmp(&right.r#type))
            .then_with(|| left.target.cmp(&right.target))
            .then_with(|| left.pillars.cmp(&right.pillars))
            .then_with(|| left.outcome.cmp(&right.outcome))
            .then_with(|| left.detail.cmp(&right.detail))
    });
    let interaction_rows = ordered_interactions
        .into_iter()
        .enumerate()
        .map(|(index, interaction)| LessonInteractionEvidence {
            id: format!("interaction-{}-{}", index + 1, interaction.r#type),
            scope: interaction.scope.clone(),
            interaction_type: interaction.r#type.clone(),
            target: interaction.target.clone(),
            outcome: interaction.outcome.clone(),
            pillars: interaction.pillars.clone(),
            detail: interaction.detail.clone(),
            assessment: "conditional_not_rescored".to_string(),
        })
        .collect::<Vec<_>>();
    let mut evidence = vec![
        LessonEvidence {
            id: "strength-same-season".to_string(),
            kind: "lesson_rule".to_string(),
            pillar: Some("月支".to_string()),
            hidden_stem_layer: Some("本氣".to_string()),
            interaction_id: None,
            description: format!("同氣得時={same_season_element}；印當令={seal_in_season}，兩者分開記錄。"),
            source_date: Some("2026-07-22".to_string()),
        },
        LessonEvidence {
            id: "strength-party-count".to_string(),
            kind: "lesson_rule".to_string(),
            pillar: None,
            hidden_stem_layer: None,
            interaction_id: None,
            description: format!(
                "得勢：八字中同黨（印、比劫）{same_party_count} 個、異黨（食傷、財、官殺）{opposing_party_count} 個（天干與地支本氣，含日主）；{}，得勢={momentum_proxy}。",
                if party_tie {
                    format!(
                        "數量相同，依「得令者為旺」看月令，月令屬{}",
                        if month_command_same_party { "同黨" } else { "異黨" }
                    )
                } else if same_party_count > opposing_party_count {
                    "同黨較多".to_string()
                } else {
                    "異黨較多".to_string()
                }
            ),
            source_date: Some("2026-09-02".to_string()),
        },
        LessonEvidence {
            id: "strength-roots".to_string(),
            kind: "chart_fact".to_string(),
            pillar: None,
            hidden_stem_layer: None,
            interaction_id: None,
            description: format!("同五行藏干根共 {} 處，保留全部層位。", facts.roots.len()),
            source_date: None,
        },
    ];
    for interaction in &interaction_rows {
        evidence.push(LessonEvidence {
            id: format!("strength-{}", interaction.id),
            kind: "chart_fact".to_string(),
            pillar: Some(interaction.pillars.clone()),
            hidden_stem_layer: None,
            interaction_id: Some(interaction.id.clone()),
            description: format!(
                "{}{}（{}，{}）；只保留關係與模型裁決，不另加扣分。",
                interaction.scope,
                interaction.interaction_type,
                interaction.target,
                interaction.outcome
            ),
            source_date: Some("2026-07-29".to_string()),
        });
    }
    let status = if classification == "依據不足" {
        "insufficient"
    } else if interaction_rows.is_empty() {
        "candidate"
    } else {
        "conditional"
    };
    let mut limitations = vec![
        "得勢只數八字的五行黨別（地支取本氣），不計干支力量大小與合沖後的變化。".to_string(),
        "表層剋洩耗與藏干出現分欄；不以藏干反向力量冒稱表層壓力。".to_string(),
    ];
    if tie_won_by_month && observation_count == 1 {
        limitations.push(
            "同黨異黨四比四、月令屬同黨：依 2026-09-02 課堂「得令者為旺」判強，即使未得時、未得地。"
                .to_string(),
        );
    }
    if !interaction_rows.is_empty() {
        limitations.push("互動後有效力量未定；保留初判並標 conditional。".to_string());
    }

    LessonStrengthAnalysis {
        title: "第二步：日主強弱".to_string(),
        conclusion: format!(
            "三項觀察命中 {observation_count} 項；表層剋洩耗家族為{}，初判{}{}。",
            if pressure_families_surface.is_empty() {
                "無".to_string()
            } else {
                pressure_families_surface.join("、")
            },
            classification,
            if tie_won_by_month && observation_count == 1 {
                "（同黨異黨四比四，得令者為旺）"
            } else {
                ""
            }
        ),
        status: status.to_string(),
        classification,
        same_season_element,
        seal_in_season,
        support_stem_count,
        support_branch_count,
        same_party_count,
        opposing_party_count,
        month_command_same_party,
        momentum_proxy,
        root_count: facts.roots.len(),
        observation_count,
        same_party_characters,
        opposing_party_characters,
        checks,
        verdict_reason,
        pressure_families_surface,
        pressure_families_hidden,
        interactions: interaction_rows,
        evidence,
        limitations,
    }
}

struct StrengthCheckInputs {
    same_season_element: bool,
    seal_in_season: bool,
    same_party_count: usize,
    opposing_party_count: usize,
    month_command_same_party: bool,
    momentum_proxy: bool,
}

/// Months in which each element is 當旺 (2026-07-22 得時 definition).
fn season_branches(element: &str) -> &'static str {
    match element {
        "木" => "寅卯",
        "火" => "巳午",
        "土" => "辰戌丑未",
        "金" => "申酉",
        _ => "亥子",
    }
}

/// Each of the eight characters labelled by position and ten god (branches by 本氣), split into
/// 同黨 and 異黨 for the 得勢 count.
fn party_characters(
    facts: &LessonFacts,
    is_same_party: &dyn Fn(&str) -> bool,
) -> (Vec<String>, Vec<String>) {
    let mut same = Vec::new();
    let mut opposing = Vec::new();
    for (index, pillar) in facts.pillars.iter().enumerate() {
        let branch_ten_god = pillar
            .hidden_stems
            .first()
            .map(|hidden| hidden.ten_god.as_str())
            .unwrap_or("");
        for (element, label) in [
            (
                pillar.stem_element.as_str(),
                format!("{}{}（{}）", STEM_LOCATION_LABELS[index], pillar.stem, pillar.stem_ten_god),
            ),
            (
                pillar.branch_main_element.as_str(),
                format!("{}{}（{}）", BRANCH_LOCATION_LABELS[index], pillar.branch, branch_ten_god),
            ),
        ] {
            if is_same_party(element) {
                same.push(label);
            } else {
                opposing.push(label);
            }
        }
    }
    (same, opposing)
}

fn strength_checks(facts: &LessonFacts, inputs: StrengthCheckInputs) -> Vec<LessonStrengthCheck> {
    let day = &facts.day_master;
    let month = &facts.month_command;
    let season_finding = if inputs.same_season_element {
        format!(
            "日主{}屬{}，生於{}月（本氣{}屬{}），月令與日主同五行 → 得令。",
            day.stem, day.element, month.branch, month.main_stem, month.element
        )
    } else if inputs.seal_in_season {
        format!(
            "日主{}屬{}，生於{}月（本氣{}屬{}，為{}）；{}要生於{}月才算得令 → 失令。月令是印（同黨），能生身，但不算得令。",
            day.stem,
            day.element,
            month.branch,
            month.main_stem,
            month.element,
            month.ten_god,
            day.element,
            season_branches(&day.element)
        )
    } else {
        format!(
            "日主{}屬{}，生於{}月（本氣{}屬{}，為{}）；{}要生於{}月才算得令 → 失令。",
            day.stem,
            day.element,
            month.branch,
            month.main_stem,
            month.element,
            month.ten_god,
            day.element,
            season_branches(&day.element)
        )
    };
    let party_finding = {
        let comparison = if inputs.same_party_count == inputs.opposing_party_count {
            format!(
                "兩黨相同，依「得令者為旺」看月令：月令{}屬{}",
                month.branch,
                if inputs.month_command_same_party { "同黨" } else { "異黨" }
            )
        } else if inputs.same_party_count > inputs.opposing_party_count {
            "同黨較多".to_string()
        } else {
            "異黨較多".to_string()
        };
        format!(
            "同黨 {} 個、異黨 {} 個；{} → {}。",
            inputs.same_party_count,
            inputs.opposing_party_count,
            comparison,
            if inputs.momentum_proxy { "得勢" } else { "失勢" }
        )
    };
    let root_finding = if facts.roots.is_empty() {
        format!("四支藏干都沒有{}（不分陰陽）→ 不得地。", day.element)
    } else {
        format!(
            "{} → 得地（共 {} 處根）。",
            facts
                .roots
                .iter()
                .map(|root| format!(
                    "{}{}藏{}（{}根，{}）",
                    root.pillar, root.branch, root.hidden_stem, root.hidden_stem_layer, root.ten_god
                ))
                .collect::<Vec<_>>()
                .join("、"),
            facts.roots.len()
        )
    };
    vec![
        LessonStrengthCheck {
            item: "得令（得時）".to_string(),
            hit: inputs.same_season_element,
            rule: "日干生在同五行當旺的月令：木寅卯、火巳午、土辰戌丑未、金申酉、水亥子。".to_string(),
            finding: season_finding,
            source_date: "2026-07-22".to_string(),
        },
        LessonStrengthCheck {
            item: "得勢".to_string(),
            hit: inputs.momentum_proxy,
            rule: "八個字（四天干含日主、四地支取本氣）分同黨（印、比劫）與異黨（食傷、財、官殺），多者勝；四比四時看月令屬哪一黨（得令者為旺）。".to_string(),
            finding: party_finding,
            source_date: "2026-09-02".to_string(),
        },
        LessonStrengthCheck {
            item: "得地".to_string(),
            hit: !facts.roots.is_empty(),
            rule: "地支藏干中有與日干同五行者即通根，不分陰陽；根以本氣最穩，其次長生，再次餘氣、墓庫。".to_string(),
            finding: root_finding,
            source_date: "2026-07-22".to_string(),
        },
    ]
}

fn verdict_reason(
    checks: &[LessonStrengthCheck],
    classification: &str,
    pressure_families_surface: &[String],
    tie_won_by_month: bool,
) -> String {
    let hits = checks
        .iter()
        .filter(|check| check.hit)
        .map(|check| check.item.trim_end_matches("（得時）"))
        .collect::<Vec<_>>();
    let marks = checks
        .iter()
        .map(|check| {
            format!(
                "{}{}",
                check.item.trim_end_matches("（得時）"),
                if check.hit { "✓" } else { "✗" }
            )
        })
        .collect::<Vec<_>>()
        .join("、");
    let rule = match classification {
        "旺極候選" => "三者兼備，天干與地支本氣都不見剋洩耗 → 旺極，入專旺格候選（2026-07-22、07-29）".to_string(),
        "旺候選" => format!(
            "三者兼備，但見{} → 旺，仍屬普通格局（2026-07-29）",
            pressure_families_surface.join("、")
        ),
        "強候選" if tie_won_by_month && hits.len() == 1 => {
            "三者只得其一，但同黨異黨四比四、月令屬同黨，依「得令者為旺」判強（2026-09-02 課堂命例）".to_string()
        }
        "強候選" if hits.contains(&"得令") => format!(
            "三者得其二（{}）→ 強（2026-07-22）。註：2026-07-29 把「得時＋得勢」「得時＋得地」也歸為旺，兩說都屬身強一方",
            hits.join("＋")
        ),
        "強候選" => format!("三者得其二（{}）→ 強；不得時但得勢得地，衰中轉強（2026-07-22、07-29）", hits.join("＋")),
        "弱候選" => format!("三者只得其一（{}）→ 弱（2026-07-29）", hits.join("")),
        "衰極候選" => "三者皆無，表層剋洩耗俱全 → 衰極，可能入從格（2026-07-22）".to_string(),
        _ => "三者皆無，但剋洩耗未全見 → 依據不足，需再看合沖會化後的五行變化（2026-07-29）".to_string(),
    };
    format!("{marks}；{rule}。")
}
