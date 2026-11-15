use super::*;

pub(super) fn build_preferences(
    facts: &LessonFacts,
    strength: &LessonStrengthAnalysis,
) -> LessonPreferenceAnalysis {
    let day_element = Element::from_name(&facts.day_master.element);
    let is_strong = matches!(
        strength.classification.as_str(),
        "強候選" | "旺候選" | "旺極候選"
    );
    let is_weak = matches!(strength.classification.as_str(), "弱候選" | "衰極候選");
    let surface = |family: &str| family_distribution(facts, family).surface_locations.clone();
    let hidden = |family: &str| family_distribution(facts, family).hidden_locations.clone();
    let mut plans = Vec::new();

    if is_strong && !surface("印").is_empty() {
        plans.push(plan(
            &day_element,
            "strong-seal-use-wealth",
            "強旺印多：財制印",
            "若按普通格論，且印確為本局主導問題，財具位置與承接能力時",
            &["財"],
            &["食傷"],
            &["比劫", "印"],
            &[],
            family_evidence_ids(facts, &["印", "財", "食傷"]),
            &["財僅出現不代表能制印", "比劫可能制財", "印可能制食傷"],
            &["2026-08-26"],
        ));
    }
    if is_strong
        && !surface("比劫").is_empty()
        && !surface("官殺").is_empty()
        && !hidden("官殺").is_empty()
    {
        plans.push(plan(
            &day_element,
            "strong-peer-use-officer",
            "強旺比劫：官殺制比",
            "若按普通格論，且比劫為主導問題，官殺可見、有同五行藏根並能制身時",
            &["官殺"],
            &["財"],
            &["印", "比劫"],
            &["食傷"],
            family_evidence_ids(facts, &["比劫", "官殺", "財"]),
            &["食傷可能制所用官殺", "印可能反生身", "有根不等於制身已成立"],
            &["2026-08-26"],
        ));
    }
    if is_strong && !surface("比劫").is_empty() && !surface("食傷").is_empty() {
        plans.push(plan(
            &day_element,
            "strong-peer-use-output",
            "強旺比劫：食傷洩比",
            "若按普通格論，且比劫為主導問題，食傷可見並有財承接時",
            &["食傷"],
            &["財"],
            &["印"],
            &[],
            family_evidence_ids(facts, &["比劫", "食傷", "財"]),
            &["印可能制食傷", "財的承接能力尚未量化"],
            &["2026-08-26"],
        ));
    }
    if is_strong
        && !hidden("財").is_empty()
        && !surface("官殺").is_empty()
        && !hidden("官殺").is_empty()
    {
        plans.push(plan(
            &day_element,
            "strong-hidden-wealth-visible-kill",
            "財藏殺顯：官殺候選",
            "若按普通格論，財藏而官殺顯露且有同五行藏根，官殺確能制身時",
            &["官殺"],
            &["財"],
            &["印"],
            &["食傷"],
            family_evidence_ids(facts, &["財", "官殺"]),
            &[
                "財藏不等於無財",
                "殺顯有根仍不代表足以制身",
                "與印多取財方案分開",
            ],
            &["2026-08-19"],
        ));
    }
    if is_strong && !hidden("官殺").is_empty() && !surface("財").is_empty() {
        plans.push(plan(
            &day_element,
            "strong-hidden-officer-visible-wealth",
            "官藏財顯：財候選",
            "若按普通格論，官殺藏支且財在天干或支本氣可見，財確能承擔耗身時",
            &["財"],
            &["食傷"],
            &["比劫", "印"],
            &[],
            family_evidence_ids(facts, &["官殺", "財", "食傷"]),
            &["支本氣顯露不冒稱透干", "財可見不代表已認證有效力量"],
            &["2026-09-02"],
        ));
    }
    if is_weak && !surface("官殺").is_empty() {
        plans.push(plan(
            &day_element,
            "weak-officer-use-seal",
            "身弱官殺：印化殺扶身",
            "若按普通格論，官殺確為主要壓力，且印能承接官殺再生日主時",
            &["印"],
            &["比劫"],
            &["財"],
            &[],
            family_evidence_ids(facts, &["官殺", "印", "比劫"]),
            &["財可能破印並生官殺", "有印不等於通路已成立"],
            &["2026-08-26"],
        ));
    }
    if is_weak && !surface("食傷").is_empty() {
        plans.push(plan(
            &day_element,
            "weak-output-use-seal",
            "身弱食傷：印制食傷扶身",
            "若按普通格論，食傷確為主要洩身壓力，且印未受制時",
            &["印"],
            &["比劫"],
            &["財", "食傷"],
            &[],
            family_evidence_ids(facts, &["食傷", "印", "比劫"]),
            &["印可能受制", "食傷生財可能加重耗身"],
            &["2026-08-26"],
        ));
    }
    if is_weak && !surface("財").is_empty() {
        plans.push(plan(
            &day_element,
            "weak-wealth-use-peer",
            "身弱財旺：比劫分財",
            "若按普通格論，財確為主要耗身壓力，比劫不受官殺制時",
            &["比劫"],
            &["印"],
            &["食傷", "官殺", "財"],
            &[],
            family_evidence_ids(facts, &["財", "比劫", "印"]),
            &["食傷可能生財", "官殺可能制比", "財可能破印"],
            &["2026-08-12", "2026-08-26"],
        ));
    }

    if plans.is_empty() && is_strong {
        plans.push(plan(
            &day_element,
            "ordinary-strong-fuyi-fallback",
            "普通扶抑：強旺待比較剋洩耗",
            "若按普通格論，且特殊格資格排除後，再比較食傷、財、官殺的實際位置、承接與受制",
            &[],
            &[],
            &[],
            &["食傷", "財", "官殺"],
            family_evidence_ids(facts, &["食傷", "財", "官殺"]),
            &[
                "未命中細分分支的完整表層位置條件",
                "特殊格資格尚未排除",
                "未認證任一方向的有效力量",
            ],
            &["2026-07-22", "2026-08-26"],
        ));
    } else if plans.is_empty() && is_weak {
        plans.push(plan(
            &day_element,
            "ordinary-weak-fuyi-fallback",
            "普通扶抑：身弱待比較印比",
            "若按普通格論，且特殊格資格排除後，再比較印、比劫的實際位置、承接與受制",
            &[],
            &[],
            &[],
            &["印", "比劫"],
            family_evidence_ids(facts, &["印", "比劫"]),
            &[
                "未命中細分分支的完整表層位置條件",
                "特殊格資格尚未排除",
                "未認證任一方向的有效力量",
            ],
            &["2026-07-22", "2026-08-26"],
        ));
    }

    // 母慈滅子（2026-11-04 講義例一）：印是病，普通身弱「用印」的方案都不適用，只留比劫方案。
    if strength.mother_overload.is_some() {
        plans = vec![plan(
            &day_element,
            "weak-seal-overload-use-peer",
            "母慈滅子：比劫洩印幫身",
            "印太多反埋日主、原局不見比劫時；運只宜比劫，財要原局有根才能疏印",
            &["比劫"],
            &[],
            &["印", "官殺"],
            &["財", "食傷"],
            family_evidence_ids(facts, &["印", "比劫", "財"]),
            &[
                "財（制印）若原局無根，印旺財折，走財運也沒用",
                "食傷洩身，日主無根時難承受",
                "官殺會再生印，加重病",
            ],
            &["2026-11-04"],
        )];
    }

    plans.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
    let no_plan_reasons = if plans.is_empty() {
        vec![if strength.classification == "依據不足" {
            "強弱零項但表層剋洩耗三家未全現，依規格不判衰極；暫無細分方案。".to_string()
        } else {
            "本盤沒有足夠表層位置證據命中八個細分分支；保留普通扶抑缺口，不固定套故事。".to_string()
        }]
    } else {
        Vec::new()
    };
    let bridge_paths = build_bridge_paths(facts);
    let medicine_notes = plans
        .iter()
        .filter_map(|item| {
            let protected = item.use_directions.first()?;
            let hindrance = item.unfavorable_directions.first()?;
            let medicine = item.favorable_directions.first();
            Some(LessonMedicineNote {
                protected_plan_rule_id: item.rule_id.clone(),
                protected_family: protected.family.clone(),
                protected_element: protected.element.clone(),
                hindrance_family: hindrance.family.clone(),
                medicine_family: medicine.map(|direction| direction.family.clone()),
                status: "conditional".to_string(),
                explanation: format!(
                    "只在方案 {} 中保護{}方向；{}是否為病、{}是否能護用尚須比較力量。",
                    item.rule_id,
                    protected.family,
                    hindrance.family,
                    medicine
                        .map(|direction| direction.family.as_str())
                        .unwrap_or("相應制衡方向")
                ),
            })
        })
        .collect::<Vec<_>>();
    let seasonal_compatibility = build_seasonal_notes(facts, &plans);
    let evidence = plans
        .iter()
        .map(|item| LessonEvidence {
            id: format!("preference-{}", item.rule_id),
            kind: "lesson_rule".to_string(),
            pillar: None,
            hidden_stem_layer: None,
            interaction_id: None,
            description: format!("{}；{}。", item.title, item.condition),
            source_date: item.sources.first().cloned(),
        })
        .collect::<Vec<_>>();
    let status = if plans.is_empty() {
        "insufficient"
    } else {
        "conditional"
    };

    LessonPreferenceAnalysis {
        title: "第三步：用神與喜忌".to_string(),
        conclusion: if plans.is_empty() {
            "條件不足，未產生細分方案；不得把空白補成固定用神。".to_string()
        } else {
            format!(
                "依實際表層與根氣位置保留 {} 個條件方案；排序不代表優先。",
                plans.len()
            )
        },
        status: status.to_string(),
        plans,
        no_plan_reasons,
        bridge_paths,
        medicine_notes,
        seasonal_compatibility,
        evidence,
        limitations: vec![
            "所有普通方案都以『若按普通格論』為適用條件，且不合併互相衝突的角色。".to_string(),
            "出現只證明位置存在，不代表該十神主導或有效力量已認證。".to_string(),
            "通關元素存在也不宣稱通關成功；病藥不覆寫上游強弱。".to_string(),
        ],
    }
}

fn plan(
    day_element: &Element,
    rule_id: &str,
    title: &str,
    condition: &str,
    use_families: &[&str],
    favorable_families: &[&str],
    unfavorable_families: &[&str],
    conditional_families: &[&str],
    evidence_ids: Vec<String>,
    rejections: &[&str],
    sources: &[&str],
) -> LessonPreferencePlan {
    let directions = |families: &[&str], role: &str| {
        families
            .iter()
            .map(|family| LessonDirection {
                family: (*family).to_string(),
                element: element_name(&family_element(day_element, family)),
                role: role.to_string(),
                reason: format!(
                    "{}家族依日主五行換算為{}。",
                    family,
                    element_name(&family_element(day_element, family))
                ),
            })
            .collect()
    };
    LessonPreferencePlan {
        rule_id: rule_id.to_string(),
        title: title.to_string(),
        applicability: "conditional".to_string(),
        condition: condition.to_string(),
        use_directions: directions(use_families, "use"),
        favorable_directions: directions(favorable_families, "favorable"),
        unfavorable_directions: directions(unfavorable_families, "unfavorable"),
        conditional_directions: directions(conditional_families, "conditional"),
        evidence_ids,
        rejections: rejections.iter().map(|item| (*item).to_string()).collect(),
        sources: sources.iter().map(|item| (*item).to_string()).collect(),
    }
}

fn build_bridge_paths(facts: &LessonFacts) -> Vec<LessonBridgePath> {
    let all_elements = facts
        .pillars
        .iter()
        .flat_map(|pillar| {
            std::iter::once(pillar.stem_element.clone()).chain(
                pillar
                    .hidden_stems
                    .iter()
                    .map(|hidden| hidden.element.clone()),
            )
        })
        .collect::<BTreeSet<_>>();
    (0..5)
        .map(|index| {
            let from = Element::from_index(index);
            let intermediary = from.get_reinforce();
            let to = from.get_restrain();
            let from_name = element_name(&from);
            let intermediary_name = element_name(&intermediary);
            let to_name = element_name(&to);
            let actual_nodes = [&from_name, &intermediary_name, &to_name]
                .into_iter()
                .filter(|name| all_elements.contains(*name))
                .cloned()
                .collect::<Vec<_>>();
            let missing_nodes = [&from_name, &intermediary_name, &to_name]
                .into_iter()
                .filter(|name| !all_elements.contains(*name))
                .cloned()
                .collect::<Vec<_>>();
            let endpoints_present =
                all_elements.contains(&from_name) && all_elements.contains(&to_name);
            let intermediary_present = all_elements.contains(&intermediary_name);
            let status = if !endpoints_present {
                "unassessed"
            } else if !intermediary_present {
                "missing_intermediary"
            } else {
                "candidate"
            };
            LessonBridgePath {
                conflict_from: from_name.clone(),
                intermediary: intermediary_name.clone(),
                conflict_to: to_name.clone(),
                path: vec![
                    from_name.clone(),
                    intermediary_name.clone(),
                    to_name.clone(),
                ],
                actual_nodes,
                missing_nodes,
                status: status.to_string(),
                explanation: match status {
                    "unassessed" => "相剋兩端未在本盤資料中同時出現，故不評估通關。".to_string(),
                    "missing_intermediary" => format!(
                        "相剋兩端存在，但缺少{}中介節點；不宣稱通關。",
                        intermediary_name
                    ),
                    _ => format!(
                        "三個節點均存在，僅列{}→{}→{}候選；位置、受制與力量尚未認證。",
                        from_name, intermediary_name, to_name
                    ),
                },
            }
        })
        .collect()
}

fn build_seasonal_notes(
    facts: &LessonFacts,
    plans: &[LessonPreferencePlan],
) -> Vec<LessonSeasonalNote> {
    let month = facts.month_command.branch.as_str();
    let (season, branches, element) = if ["亥", "子", "丑"].contains(&month) {
        ("winter", vec!["亥", "子", "丑"], "火")
    } else if ["巳", "午", "未"].contains(&month) {
        ("summer", vec!["巳", "午", "未"], "水")
    } else {
        return vec![LessonSeasonalNote {
            plan_rule_id: None,
            season: "outside_bounded_scope".to_string(),
            month_branches: Vec::new(),
            status: "unassessed".to_string(),
            element: None,
            explanation: "月支不在首版三冬／三夏範圍；調候未評估，不套用其他課例。".to_string(),
        }];
    };
    if plans.is_empty() {
        return vec![LessonSeasonalNote {
            plan_rule_id: None,
            season: season.to_string(),
            month_branches: branches.into_iter().map(str::to_string).collect(),
            status: "unassessed".to_string(),
            element: Some(element.to_string()),
            explanation: format!("尚無可保護的扶抑方案，首版不因季節自動補入{}。", element),
        }];
    }
    plans
        .iter()
        .map(|plan| {
            let matches = |directions: &[LessonDirection]| {
                directions
                    .iter()
                    .any(|direction| direction.element == element)
            };
            let (status, explanation) = if matches(&plan.unfavorable_directions) {
                (
                    "conflict",
                    format!(
                        "{}方案將{}列為忌向；調候不得覆蓋此扶抑衝突。",
                        plan.rule_id, element
                    ),
                )
            } else if matches(&plan.conditional_directions) {
                (
                    "unresolved",
                    format!(
                        "{}方案將{}列為條件方向；其是否制所用尚未解決，不作相容。",
                        plan.rule_id, element
                    ),
                )
            } else if matches(&plan.use_directions) || matches(&plan.favorable_directions) {
                (
                    "conditional_compatible",
                    format!(
                        "{}方案已將{}列入用／喜方向；可比較寒暖，但仍須檢查承接與受制。",
                        plan.rule_id, element
                    ),
                )
            } else {
                (
                    "unassessed",
                    format!(
                        "{}方案未給出{}方向，首版不因季節自動補入。",
                        plan.rule_id, element
                    ),
                )
            };
            LessonSeasonalNote {
                plan_rule_id: Some(plan.rule_id.clone()),
                season: season.to_string(),
                month_branches: branches
                    .iter()
                    .map(|branch| (*branch).to_string())
                    .collect(),
                status: status.to_string(),
                element: Some(element.to_string()),
                explanation,
            }
        })
        .collect()
}
