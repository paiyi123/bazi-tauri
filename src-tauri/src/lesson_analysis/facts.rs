use super::*;

pub(super) fn build_facts(natal_cycles: &[SixtyCycle; 4]) -> LessonFacts {
    let day_stem = natal_cycles[2].get_heaven_stem();
    let day_element = day_stem.get_element();
    let mut evidence = Vec::new();
    let mut roots = Vec::new();
    let mut pillars = Vec::new();

    for (pillar_index, cycle) in natal_cycles.iter().enumerate() {
        let stem = cycle.get_heaven_stem();
        let branch = cycle.get_earth_branch();
        let stem_name = stem.get_name();
        let branch_name = branch.get_name();
        let hidden_stems = branch
            .get_hide_heaven_stems()
            .into_iter()
            .enumerate()
            .map(|(hidden_index, hidden)| {
                let hidden_stem = hidden.get_heaven_stem();
                let hidden_name = hidden_stem.get_name();
                let hidden_element = hidden_stem.get_element();
                let layer = hidden_layer(hidden.get_type()).to_string();
                let evidence_id = format!("fact-hidden-{}-{}", pillar_index + 1, hidden_index + 1);
                evidence.push(LessonEvidence {
                    id: evidence_id.clone(),
                    kind: "chart_fact".to_string(),
                    pillar: Some(BRANCH_LOCATION_LABELS[pillar_index].to_string()),
                    hidden_stem_layer: Some(layer.clone()),
                    interaction_id: None,
                    description: format!(
                        "{}{}藏{}（{}、{}）。",
                        BRANCH_LOCATION_LABELS[pillar_index],
                        branch_name,
                        hidden_name,
                        layer,
                        element_name(&hidden_element)
                    ),
                    source_date: None,
                });
                if hidden_element == day_element {
                    roots.push(LessonRootFact {
                        pillar: BRANCH_LOCATION_LABELS[pillar_index].to_string(),
                        branch: branch_name.clone(),
                        hidden_stem: hidden_name.clone(),
                        hidden_stem_layer: layer.clone(),
                        element: element_name(&hidden_element),
                        ten_god: ten_god_name(&day_stem, &hidden_stem),
                        root_kind: if hidden_stem == day_stem {
                            "same_stem"
                        } else {
                            "same_element"
                        }
                        .to_string(),
                        evidence_id,
                    });
                }
                LessonHiddenStemFact {
                    stem: hidden_name,
                    element: element_name(&hidden_element),
                    ten_god: ten_god_name(&day_stem, &hidden_stem),
                    layer,
                }
            })
            .collect::<Vec<_>>();
        let stem_evidence_id = format!("fact-stem-{}", pillar_index + 1);
        evidence.push(LessonEvidence {
            id: stem_evidence_id,
            kind: "chart_fact".to_string(),
            pillar: Some(STEM_LOCATION_LABELS[pillar_index].to_string()),
            hidden_stem_layer: None,
            interaction_id: None,
            description: format!(
                "{}為{}，五行{}，相對日主為{}。",
                STEM_LOCATION_LABELS[pillar_index],
                stem_name,
                element_name(&stem.get_element()),
                if pillar_index == 2 {
                    "日元".to_string()
                } else {
                    ten_god_name(&day_stem, &stem)
                }
            ),
            source_date: None,
        });
        pillars.push(LessonPillarFact {
            pillar: PILLAR_LABELS[pillar_index].to_string(),
            gan_zhi: cycle.get_name(),
            stem: stem_name,
            stem_element: element_name(&stem.get_element()),
            stem_ten_god: if pillar_index == 2 {
                "日元".to_string()
            } else {
                ten_god_name(&day_stem, &stem)
            },
            branch: branch_name,
            branch_main_element: element_name(&branch.get_element()),
            hidden_stems,
        });
    }

    let month_branch = natal_cycles[1].get_earth_branch();
    let month_hidden = month_branch.get_hide_heaven_stems();
    let month_main = month_hidden[0].get_heaven_stem();
    let mut transparencies = Vec::new();
    for (hidden_index, hidden) in month_hidden.iter().enumerate() {
        let hidden_stem = hidden.get_heaven_stem();
        let visible_locations = [0usize, 1, 3]
            .into_iter()
            .filter(|index| natal_cycles[*index].get_heaven_stem() == hidden_stem)
            .map(|index| STEM_LOCATION_LABELS[index].to_string())
            .collect::<Vec<_>>();
        let evidence_id = format!("fact-transparency-{}", hidden_index + 1);
        evidence.push(LessonEvidence {
            id: evidence_id.clone(),
            kind: "chart_fact".to_string(),
            pillar: Some("月支".to_string()),
            hidden_stem_layer: Some(hidden_layer(hidden.get_type()).to_string()),
            interaction_id: None,
            description: if visible_locations.is_empty() {
                format!("月令藏干{}未在年、月、時干透出。", hidden_stem.get_name())
            } else {
                format!(
                    "月令藏干{}透於{}。",
                    hidden_stem.get_name(),
                    visible_locations.join("、")
                )
            },
            source_date: None,
        });
        transparencies.push(LessonTransparencyFact {
            hidden_stem: hidden_stem.get_name(),
            hidden_stem_layer: hidden_layer(hidden.get_type()).to_string(),
            ten_god: ten_god_name(&day_stem, &hidden_stem),
            visible_locations,
            evidence_id,
        });
    }

    LessonFacts {
        title: "排盤事實".to_string(),
        conclusion: format!(
            "日主{}屬{}，月令{}本氣{}屬{}；根氣與透干均保留原柱位及藏干層位。",
            day_stem.get_name(),
            element_name(&day_element),
            month_branch.get_name(),
            month_main.get_name(),
            element_name(&month_main.get_element())
        ),
        pillars,
        day_master: LessonDayMasterFact {
            stem: day_stem.get_name(),
            element: element_name(&day_element),
        },
        month_command: LessonMonthCommandFact {
            branch: month_branch.get_name(),
            main_stem: month_main.get_name(),
            element: element_name(&month_main.get_element()),
            ten_god: ten_god_name(&day_stem, &month_main),
        },
        transparencies,
        roots,
        ten_god_distribution: build_family_distribution(natal_cycles, &day_stem),
        evidence,
        limitations: vec![
            "藏干事實不因合沖而刪除；互動後是否仍有效另在強弱步驟標為待定。".to_string(),
        ],
    }
}

fn build_family_distribution(
    natal_cycles: &[SixtyCycle; 4],
    day_stem: &HeavenStem,
) -> Vec<LessonFamilyDistribution> {
    let day_element = day_stem.get_element();
    FAMILY_ORDER
        .iter()
        .map(|family| {
            let mut surface_locations = Vec::new();
            let mut hidden_locations = Vec::new();
            let mut surface_evidence_ids = Vec::new();
            let mut hidden_evidence_ids = Vec::new();
            for (index, cycle) in natal_cycles.iter().enumerate() {
                if index != 2 {
                    let stem = cycle.get_heaven_stem();
                    if family_for_element(&day_element, &stem.get_element()) == *family {
                        surface_locations.push(format!(
                            "{}{}",
                            STEM_LOCATION_LABELS[index],
                            stem.get_name()
                        ));
                        surface_evidence_ids.push(format!("fact-stem-{}", index + 1));
                    }
                }
                for (hidden_index, hidden) in cycle
                    .get_earth_branch()
                    .get_hide_heaven_stems()
                    .into_iter()
                    .enumerate()
                {
                    let hidden_stem = hidden.get_heaven_stem();
                    if family_for_element(&day_element, &hidden_stem.get_element()) == *family {
                        let location = format!(
                            "{}{}{}{}",
                            BRANCH_LOCATION_LABELS[index],
                            cycle.get_earth_branch().get_name(),
                            hidden_layer(hidden.get_type()),
                            hidden_stem.get_name()
                        );
                        if hidden.get_type() == HideHeavenStemType::MAIN {
                            surface_locations.push(location.clone());
                            surface_evidence_ids.push(format!(
                                "fact-hidden-{}-{}",
                                index + 1,
                                hidden_index + 1
                            ));
                        }
                        hidden_locations.push(location);
                        hidden_evidence_ids.push(format!(
                            "fact-hidden-{}-{}",
                            index + 1,
                            hidden_index + 1
                        ));
                    }
                }
            }
            LessonFamilyDistribution {
                family: (*family).to_string(),
                element: element_name(&family_element(&day_element, family)),
                surface_locations,
                hidden_locations,
                surface_evidence_ids,
                hidden_evidence_ids,
            }
        })
        .collect()
}
