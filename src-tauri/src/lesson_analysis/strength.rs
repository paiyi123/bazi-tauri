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
    let momentum_proxy = support_stem_count >= 2 || support_branch_count >= 2;
    let rooted = !facts.roots.is_empty();
    let observation_count =
        usize::from(same_season_element) + usize::from(momentum_proxy) + usize::from(rooted);
    let pressure_families_surface = pressure_families(facts, true);
    let pressure_families_hidden = pressure_families(facts, false);
    let classification = match observation_count {
        3 if pressure_families_surface.is_empty() => "旺極候選",
        3 => "旺候選",
        2 => "強候選",
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
            id: "strength-momentum-proxy".to_string(),
            kind: "engineering_proxy".to_string(),
            pillar: None,
            hidden_stem_layer: None,
            interaction_id: None,
            description: format!(
                "得勢工程暫估：supportStemCount={support_stem_count}、supportBranchCount={support_branch_count}；任一達 2 即成立，結果={momentum_proxy}。"
            ),
            source_date: Some("2026-07-22".to_string()),
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
        "supportStemCount >= 2 OR supportBranchCount >= 2 是工程候選門檻，不是老師數值定義。"
            .to_string(),
        "表層剋洩耗與藏干出現分欄；不以藏干反向力量冒稱表層壓力。".to_string(),
    ];
    if !interaction_rows.is_empty() {
        limitations.push("互動後有效力量未定；保留初判並標 conditional。".to_string());
    }

    LessonStrengthAnalysis {
        title: "第二步：日主強弱".to_string(),
        conclusion: format!(
            "三項觀察命中 {observation_count} 項；表層剋洩耗家族為{}，初判{}。",
            if pressure_families_surface.is_empty() {
                "無".to_string()
            } else {
                pressure_families_surface.join("、")
            },
            classification
        ),
        status: status.to_string(),
        classification,
        same_season_element,
        seal_in_season,
        support_stem_count,
        support_branch_count,
        momentum_proxy,
        root_count: facts.roots.len(),
        observation_count,
        pressure_families_surface,
        pressure_families_hidden,
        interactions: interaction_rows,
        evidence,
        limitations,
    }
}
