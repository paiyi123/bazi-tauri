use super::*;

fn chart(names: [&str; 4]) -> [SixtyCycle; 4] {
    names.map(SixtyCycle::from_name)
}

fn reference() -> LessonModelReference {
    LessonModelReference {
        ge_ju: "App格局".to_string(),
        strength_label: "偏強".to_string(),
        use_elements: vec!["木".to_string()],
        special_pattern: None,
        special_category: None,
    }
}

#[test]
fn three_step_facts_keep_hidden_layers_and_exclude_day_stem_transparency() {
    let result = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);

    assert_eq!(result.version, "classroom-2026-10-v2");
    assert_eq!(result.facts.day_master.stem, "庚");
    assert_eq!(result.facts.month_command.main_stem, "戊");
    assert_eq!(
        result.facts.pillars[1]
            .hidden_stems
            .iter()
            .map(|item| (item.stem.as_str(), item.layer.as_str()))
            .collect::<Vec<_>>(),
        vec![("戊", "本氣"), ("乙", "中氣"), ("癸", "餘氣")]
    );
    assert_eq!(result.pattern.ordinary_result, None);
    assert_eq!(
        result
            .pattern
            .ordinary_candidates
            .iter()
            .map(|item| (item.pattern.as_str(), item.transparent_locations.as_slice()))
            .collect::<Vec<_>>(),
        vec![
            ("偏印格", ["年干".to_string()].as_slice()),
            ("傷官格", ["時干".to_string()].as_slice()),
        ]
    );
    assert!(!result.pattern.conclusion.contains("直輸四柱分析"));
}

#[test]
fn strength_separates_same_season_from_seal_and_keeps_support_boundary_literal() {
    let same = analyze(&chart(["壬子", "辛亥", "壬子", "辛亥"]), &reference(), &[]);
    assert!(same.strength.same_season_element);
    assert!(!same.strength.seal_in_season);
    assert_eq!(same.strength.support_stem_count, 3);
    assert!(same.strength.momentum_proxy);
    assert_eq!(same.strength.classification, "旺極候選");
    assert!(same
        .strength
        .pressure_families_hidden
        .contains(&"食傷".to_string()));

    let seal = analyze(&chart(["甲寅", "丁亥", "甲午", "丙寅"]), &reference(), &[]);
    assert!(!seal.strength.same_season_element);
    assert!(seal.strength.seal_in_season);
    assert_eq!(seal.strength.support_stem_count, 1);
    assert!(seal.strength.support_branch_count >= 2);
    assert!(seal.strength.momentum_proxy);
}

#[test]
fn momentum_counts_eight_characters_by_party_and_breaks_ties_by_month_command() {
    // 2026-07-22 例6：同黨六、異黨二 → 得勢；得地，衰中轉強。
    let majority = analyze(&chart(["甲寅", "癸酉", "甲子", "丁卯"]), &reference(), &[]);
    assert_eq!(majority.strength.same_party_count, 6);
    assert_eq!(majority.strength.opposing_party_count, 2);
    assert!(majority.strength.momentum_proxy);
    assert_eq!(majority.strength.classification, "強候選");

    // 2026-09-02 學員命例二：同黨三、異黨五 → 只得地，身弱。
    let minority = analyze(&chart(["丁丑", "癸丑", "壬申", "甲辰"]), &reference(), &[]);
    assert_eq!(minority.strength.same_party_count, 3);
    assert!(!minority.strength.momentum_proxy);
    assert_eq!(minority.strength.classification, "弱候選");

    // Two visible support stems no longer count as 得勢 when the other party is larger.
    let stems_only = analyze(&chart(["甲午", "丁丑", "甲午", "甲戌"]), &reference(), &[]);
    assert_eq!(stems_only.strength.support_stem_count, 2);
    assert_eq!(stems_only.strength.same_party_count, 3);
    assert!(!stems_only.strength.momentum_proxy);
}

#[test]
fn four_to_four_tie_goes_to_the_party_holding_the_month_command() {
    // 2026-09-02 學員命例一：金水對木火四比四，金得令 → 身強，雖然未得時、未得地。
    let source = analyze(&chart(["甲寅", "癸酉", "壬午", "癸卯"]), &reference(), &[]);
    assert_eq!(source.strength.same_party_count, 4);
    assert_eq!(source.strength.opposing_party_count, 4);
    assert!(source.strength.month_command_same_party);
    assert!(source.strength.momentum_proxy);
    assert!(!source.strength.same_season_element);
    assert_eq!(source.strength.root_count, 0);
    assert_eq!(source.strength.classification, "強候選");
    assert!(source.strength.conclusion.contains("得令者為旺"));
    // 官藏財顯：the classroom's 財 plan is among the strong-side plans.
    assert!(source
        .preferences
        .plans
        .iter()
        .any(|plan| plan.rule_id == "strong-hidden-officer-visible-wealth"));

    // 2026-09-09 兩神相剋：四土四水，水得令 → 財旺身弱。
    let water_month = analyze(&chart(["戊戌", "癸亥", "戊戌", "癸亥"]), &reference(), &[]);
    assert!(!water_month.strength.month_command_same_party);
    assert!(!water_month.strength.momentum_proxy);
    assert_eq!(water_month.strength.classification, "弱候選");

    // 2026-09-09 兩神相剋：四木四土，木得令 → 建祿身旺。
    let wood_month = analyze(&chart(["乙未", "己卯", "乙未", "己卯"]), &reference(), &[]);
    assert!(wood_month.strength.momentum_proxy);
    assert_eq!(wood_month.strength.classification, "旺候選");
}

#[test]
fn no_transparency_falls_back_in_month_hidden_stem_order() {
    let result = analyze(&chart(["甲子", "丁丑", "丙寅", "壬辰"]), &reference(), &[]);

    assert_eq!(result.pattern.ordinary_candidates.len(), 3);
    assert!(result
        .pattern
        .ordinary_candidates
        .iter()
        .all(|candidate| candidate.transparent_locations.is_empty()));
    assert_eq!(
        result.pattern.ordinary_candidates[0].month_hidden_stem,
        "己"
    );
    assert_eq!(
        result.pattern.ordinary_candidates[0].hidden_stem_layer,
        "本氣"
    );
    assert_eq!(result.pattern.ordinary_candidates[0].selection, "selected");
    assert_eq!(
        result.pattern.ordinary_result,
        Some(result.pattern.ordinary_candidates[0].pattern.clone())
    );
}

#[test]
fn zero_observations_need_all_three_surface_pressure_families_for_extreme_weak() {
    let partial = analyze(&chart(["戊戌", "己丑", "甲午", "丁巳"]), &reference(), &[]);
    assert_eq!(partial.strength.observation_count, 0);
    assert_eq!(partial.strength.classification, "依據不足");
    assert_eq!(partial.strength.status, "insufficient");

    let full = analyze(&chart(["丙戌", "己丑", "甲午", "辛酉"]), &reference(), &[]);
    assert_eq!(full.strength.observation_count, 0);
    assert_eq!(
        full.strength.pressure_families_surface,
        vec!["食傷", "財", "官殺"]
    );
    assert_eq!(full.strength.classification, "衰極候選");
}

#[test]
fn interactions_preserve_positions_and_make_strength_conditional_without_pruning_roots() {
    let interaction = QuantModelInteraction {
        scope: "地支".to_string(),
        r#type: "沖".to_string(),
        target: "寅申".to_string(),
        outcome: "互沖".to_string(),
        pillars: "月支、日支".to_string(),
        detail: "原模型互動證據".to_string(),
    };
    let without = analyze(&chart(["丙子", "丙申", "庚寅", "丁亥"]), &reference(), &[]);
    let with = analyze(
        &chart(["丙子", "丙申", "庚寅", "丁亥"]),
        &reference(),
        &[interaction],
    );

    assert_eq!(with.strength.status, "conditional");
    assert_eq!(with.strength.interactions[0].pillars, "月支、日支");
    assert_eq!(with.strength.interactions[0].target, "寅申");
    assert_eq!(with.strength.root_count, without.strength.root_count);
    assert_eq!(
        with.strength.classification,
        without.strength.classification
    );
}

#[test]
fn references_change_comparison_but_not_classroom_core() {
    let cycles = chart(["丁卯", "丙午", "己酉", "癸酉"]);
    let a = analyze(&cycles, &reference(), &[]);
    let b = analyze(
        &cycles,
        &LessonModelReference {
            ge_ju: "完全不同".to_string(),
            strength_label: "偏弱".to_string(),
            use_elements: vec!["水".to_string()],
            special_pattern: Some("既有特殊格".to_string()),
            special_category: Some("從格/專旺".to_string()),
        },
        &[],
    );

    assert_eq!(
        serde_json::to_value(a.facts).unwrap(),
        serde_json::to_value(b.facts).unwrap()
    );
    assert_eq!(
        serde_json::to_value(a.strength).unwrap(),
        serde_json::to_value(b.strength).unwrap()
    );
    assert_eq!(
        serde_json::to_value(a.preferences).unwrap(),
        serde_json::to_value(b.preferences).unwrap()
    );
    assert_ne!(
        serde_json::to_value(a.comparison).unwrap(),
        serde_json::to_value(b.comparison).unwrap()
    );
}

#[test]
fn strength_comparison_requires_recognized_categories_on_both_sides() {
    let weak_chart = chart(["丁丑", "癸丑", "壬申", "甲辰"]);
    let unrecognized_app = analyze(
        &weak_chart,
        &LessonModelReference {
            strength_label: "依據不足".to_string(),
            ..reference()
        },
        &[],
    );
    assert_eq!(unrecognized_app.strength.classification, "弱候選");
    assert_eq!(
        unrecognized_app.comparison.strength.agreement_status,
        "not_comparable"
    );

    let insufficient_lesson = analyze(
        &chart(["戊戌", "己丑", "甲午", "丁巳"]),
        &LessonModelReference {
            strength_label: "偏弱".to_string(),
            ..reference()
        },
        &[],
    );
    assert_eq!(insufficient_lesson.strength.classification, "依據不足");
    assert_eq!(
        insufficient_lesson.comparison.strength.agreement_status,
        "not_comparable"
    );
}

#[test]
fn strong_chart_without_a_detailed_branch_gets_a_guarded_fuyi_fallback() {
    let result = analyze(&chart(["甲寅", "甲寅", "甲寅", "甲寅"]), &reference(), &[]);
    let fallback = result
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "ordinary-strong-fuyi-fallback")
        .expect("a recognized strong category must not leave the ordinary direction blank");

    assert_eq!(result.strength.classification, "旺極候選");
    assert!(result.pattern.special_review.required);
    assert!(fallback.condition.contains("若按普通格論"));
    assert!(fallback.condition.contains("特殊格資格排除後"));
    assert!(fallback.use_directions.is_empty());
    assert!(fallback.favorable_directions.is_empty());
    assert_eq!(
        fallback
            .conditional_directions
            .iter()
            .map(|direction| direction.family.as_str())
            .collect::<Vec<_>>(),
        vec!["食傷", "財", "官殺"]
    );
    assert!(fallback
        .rejections
        .iter()
        .any(|gap| gap.contains("未命中細分分支")));
}

#[test]
fn seasonal_assessment_stays_plan_specific_for_competing_roles() {
    let result = analyze(&chart(["庚申", "丙午", "庚申", "癸未"]), &reference(), &[]);
    let notes = serde_json::to_value(&result.preferences.seasonal_compatibility).unwrap();
    let notes = notes
        .as_array()
        .expect("seasonal notes should serialize as rows");
    let officer = notes
        .iter()
        .find(|note| note["planRuleId"] == "strong-peer-use-officer")
        .expect("officer plan needs its own seasonal assessment");
    let output = notes
        .iter()
        .find(|note| note["planRuleId"] == "strong-peer-use-output")
        .expect("output plan needs its own seasonal assessment");

    assert_eq!(officer["element"], "水");
    assert_eq!(officer["status"], "unresolved");
    assert_eq!(output["element"], "水");
    assert_eq!(output["status"], "conditional_compatible");
    assert_ne!(officer["explanation"], output["explanation"]);
}

#[test]
fn competing_plans_stay_separate_and_each_has_an_ordinary_pattern_guard() {
    let result = analyze(&chart(["丁卯", "丙午", "己酉", "癸酉"]), &reference(), &[]);

    assert!(result.preferences.plans.len() >= 2);
    assert!(
        result
            .preferences
            .plans
            .iter()
            .all(|plan| plan.applicability == "conditional"
                && plan.condition.contains("若按普通格論"))
    );
    assert!(result
        .preferences
        .plans
        .windows(2)
        .all(|pair| pair[0].rule_id <= pair[1].rule_id));
    assert!(result
        .preferences
        .medicine_notes
        .iter()
        .all(|note| !note.protected_plan_rule_id.is_empty() && !note.protected_family.is_empty()));
}

#[test]
fn preference_evidence_ids_resolve_to_real_chart_fact_evidence() {
    let result = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);
    let fact_ids = result
        .facts
        .evidence
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    assert!(result.preferences.plans.iter().all(|plan| {
        !plan.evidence_ids.is_empty()
            && plan
                .evidence_ids
                .iter()
                .all(|evidence_id| fact_ids.contains(evidence_id.as_str()))
    }));
}

#[test]
fn all_eight_ordinary_preference_branches_are_reachable_from_chart_facts() {
    let mut rule_ids = BTreeSet::new();
    for names in [
        ["丁卯", "丙午", "己酉", "癸酉"],
        ["戊申", "丙辰", "庚午", "癸未"],
        ["丙戌", "己丑", "甲午", "辛酉"],
    ] {
        rule_ids.extend(
            analyze(&chart(names), &reference(), &[])
                .preferences
                .plans
                .into_iter()
                .map(|plan| plan.rule_id),
        );
    }

    assert_eq!(
        rule_ids,
        [
            "strong-hidden-officer-visible-wealth",
            "strong-hidden-wealth-visible-kill",
            "strong-peer-use-officer",
            "strong-peer-use-output",
            "strong-seal-use-wealth",
            "weak-officer-use-seal",
            "weak-output-use-seal",
            "weak-wealth-use-peer",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    );
}

#[test]
fn wealth_hidden_kill_visible_and_officer_hidden_wealth_visible_use_distinct_evidence() {
    let kill = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);
    let kill_plan = kill
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "strong-hidden-wealth-visible-kill")
        .expect("kill plan should use hidden wealth and visible rooted kill");
    assert!(kill_plan
        .evidence_ids
        .contains(&"fact-hidden-2-2".to_string()));
    assert!(kill_plan.evidence_ids.contains(&"fact-stem-2".to_string()));

    let wealth = analyze(&chart(["壬子", "癸亥", "壬申", "丙午"]), &reference(), &[]);
    let wealth_plan = wealth
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "strong-hidden-officer-visible-wealth")
        .expect("wealth plan should use hidden officer and visible wealth");
    assert!(wealth_plan
        .evidence_ids
        .contains(&"fact-hidden-3-3".to_string()));
    assert!(wealth_plan
        .evidence_ids
        .contains(&"fact-stem-4".to_string()));
}

#[test]
fn special_guards_include_both_specialized_versions_and_follow_child_exception() {
    let result = analyze(&chart(["壬子", "辛亥", "壬子", "辛亥"]), &reference(), &[]);

    assert!(result.pattern.special_review.required);
    assert_eq!(
        result
            .pattern
            .special_review
            .specialized_use_versions
            .iter()
            .map(|item| item.source_date.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-07-08", "2026-09-02"]
    );
    assert!(result.sources.iter().any(|source| {
        source.date == "2026-07-08"
            && source.rule_ids == vec!["specialized-use-version".to_string()]
    }));
    assert!(result
        .pattern
        .special_review
        .follow_child_peer_exception
        .contains("比劫"));
    assert!(result
        .preferences
        .plans
        .iter()
        .all(|plan| plan.condition.contains("若按普通格論")));
}

#[test]
fn rootless_special_review_still_requires_no_visible_branch_support() {
    let result = analyze(&chart(["丙子", "丙子", "甲午", "丙戌"]), &reference(), &[]);

    assert!(result.facts.roots.is_empty());
    assert_eq!(result.strength.support_stem_count, 0);
    assert_eq!(result.strength.support_branch_count, 2);
    assert!(!result.pattern.special_review.required);
}

#[test]
fn bridge_contract_has_five_paths_and_winter_never_invents_fire_direction() {
    let result = analyze(&chart(["甲午", "丙子", "庚寅", "丁亥"]), &reference(), &[]);

    assert_eq!(result.preferences.bridge_paths.len(), 5);
    assert!(result
        .preferences
        .bridge_paths
        .iter()
        .all(|path| path.path.len() == 3));
    let fire_is_use_or_favorable = result.preferences.plans.iter().any(|plan| {
        plan.use_directions
            .iter()
            .chain(plan.favorable_directions.iter())
            .any(|direction| direction.element == "火")
    });
    assert!(!fire_is_use_or_favorable);
    assert!(result
        .preferences
        .seasonal_compatibility
        .iter()
        .all(|note| note.plan_rule_id.is_some()
            && note.element.as_deref() == Some("火")
            && note.status != "conditional_compatible"));
    assert!(result
        .preferences
        .seasonal_compatibility
        .iter()
        .any(|note| note.status == "conflict"));

    let missing_bridge = analyze(&chart(["壬子", "丙午", "壬子", "丙午"]), &reference(), &[]);
    let water_to_fire = missing_bridge
        .preferences
        .bridge_paths
        .iter()
        .find(|path| path.conflict_from == "水" && path.conflict_to == "火")
        .expect("water-controls-fire bridge should be one of the five mappings");
    assert_eq!(water_to_fire.intermediary, "木");
    assert_eq!(water_to_fire.status, "missing_intermediary");
    assert_eq!(water_to_fire.actual_nodes, vec!["水", "火"]);
    assert_eq!(water_to_fire.missing_nodes, vec!["木"]);
}

#[test]
fn strength_checks_spell_out_each_observation_and_the_verdict() {
    // 庚生戌月：失令（月令為印），同黨五比異黨三得勢，兩戌藏辛得地 → 強候選。
    let result = analyze(&chart(["庚戌", "丙戌", "庚寅", "己卯"]), &reference(), &[]);
    let strength = &result.strength;
    assert_eq!(strength.classification, "強候選");
    assert_eq!(
        strength.checks.iter().map(|check| (check.item.as_str(), check.hit)).collect::<Vec<_>>(),
        vec![("得令（得時）", false), ("得勢", true), ("得地", true)]
    );
    assert!(strength.checks[0].finding.contains("申酉"));
    assert!(strength.checks[0].finding.contains("月令是印"));
    assert_eq!(
        strength.same_party_characters,
        vec!["年干庚（比肩）", "年支戌（偏印）", "月支戌（偏印）", "日干庚（日元）", "時干己（正印）"]
    );
    assert_eq!(
        strength.opposing_party_characters,
        vec!["月干丙（七殺）", "日支寅（偏財）", "時支卯（正財）"]
    );
    assert!(strength.checks[2].finding.contains("年支戌藏辛"));
    assert!(strength.verdict_reason.starts_with("得令✗、得勢✓、得地✓"));
    assert!(strength.verdict_reason.contains("得勢＋得地"));

    // 四比四、月令屬同黨：說明要點出「得令者為旺」。
    let source = analyze(&chart(["甲寅", "癸酉", "壬午", "癸卯"]), &reference(), &[]);
    assert!(source.strength.checks[1].finding.contains("得令者為旺"));
    assert!(source.strength.verdict_reason.contains("四比四"));
}

// ---- 2026-10-07、10-14：天干生剋合化（講義第二十三章例題） ----

mod stem_interaction_cases {
    use super::super::stem_interactions::{classify_combinations, classify_relations, Favor};
    use super::*;

    /// 老師對該例的喜忌（只列有用到的五行）。
    fn favor(entries: &'static [(&'static str, Favor)]) -> impl Fn(&str) -> Favor {
        move |element: &str| {
            entries
                .iter()
                .find(|(name, _)| *name == element)
                .map(|(_, favor)| *favor)
                .unwrap_or(Favor::Neutral)
        }
    }

    fn combos(
        names: [&str; 4],
        entries: &'static [(&'static str, Favor)],
    ) -> Vec<LessonStemCombination> {
        classify_combinations(&chart(names), &favor(entries))
    }

    fn find<'a>(list: &'a [LessonStemCombination], stems: &str, kind: &str) -> &'a LessonStemCombination {
        list.iter()
            .find(|item| item.stems == stems && item.kind == kind)
            .unwrap_or_else(|| panic!("找不到 {stems} {kind}：{list:#?}"))
    }

    use Favor::{Bad, Good};

    #[test]
    fn he_er_bu_he_blocked_by_control_or_gap() {
        // 合而不合例一：甲剋戊（寅卯辰會木，戊坐辰根被帶走），戊不敢合癸。
        let list = combos(["甲寅", "戊辰", "癸卯", "癸亥"], &[]);
        let blocked = list.iter().find(|item| item.id == "stem-combination-2-3").unwrap();
        assert_eq!(blocked.kind_label, "合而不合");
        assert!(blocked.reason.contains("年干甲"), "{}", blocked.reason);
        assert!(blocked.reason.contains("寅卯辰"), "{}", blocked.reason);

        // 合而不合例二：丁壬中間隔著癸水。
        let list = combos(["丁卯", "癸卯", "壬申", "庚子"], &[]);
        let gap = find(&list, "丁壬", "blocked");
        assert_eq!(gap.scope, "隔位相合");
        assert!(gap.reason.contains("月干癸"));

        // 合而不化例二：日干己剋月干癸，但癸坐亥有根，仍論合。
        let list = combos(["戊午", "癸亥", "己巳", "己巳"], &[("水", Good), ("土", Bad)]);
        assert_eq!(find(&list, "戊癸", "bound").direction.as_deref(), Some("合去"));
    }

    #[test]
    fn he_er_bu_hua_stronger_stem_binds_weaker() {
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, &str, &str); 5] = [
            // 例一 庚日身強用官：丁（官）被壬絆住 → 減吉
            (["壬辰", "丁未", "庚申", "甲申"], &[("火", Good), ("土", Bad), ("金", Bad)], "壬丁", "月干丁", "bad"),
            // 例二 己日身強用財：癸（財）被戊絆住 → 損財
            (["戊午", "癸亥", "己巳", "己巳"], &[("水", Good), ("土", Bad)], "戊癸", "月干癸", "bad"),
            // 例三 甲日身弱：庚強，絆住乙（劫財，喜）→ 減吉
            (["乙亥", "庚辰", "甲申", "癸酉"], &[("木", Good), ("水", Good), ("金", Bad)], "乙庚", "年干乙", "bad"),
            // 例四 甲日身強用殺：木強，乙絆住庚（殺，喜）→ 減吉
            (["乙亥", "庚辰", "甲寅", "丁卯"], &[("金", Good), ("木", Bad)], "乙庚", "月干庚", "bad"),
            // 例五 戊日身弱用印：丙合住辛（傷官，忌）→ 減凶
            (["丙午", "辛卯", "戊寅", "乙卯"], &[("火", Good), ("土", Good), ("金", Bad), ("木", Bad)], "丙辛", "月干辛", "good"),
        ];
        for (names, entries, stems, bound, verdict) in cases {
            let list = combos(names, entries);
            let item = find(&list, stems, "bound");
            assert_eq!(item.kind_label, "合而不化", "{names:?}");
            assert_eq!(item.direction.as_deref(), Some("合去"), "{names:?}");
            assert_eq!(item.bound_stems.len(), 1, "{names:?} {item:#?}");
            assert!(item.bound_stems[0].starts_with(bound), "{names:?} {item:#?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
    }

    #[test]
    fn he_lai_he_qu_follow_position_and_preferences() {
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, &str, &str); 6] = [
            // 例一 丙日身強用殺：年壬（殺）被丁合去 → 損吉
            (["壬辰", "丁未", "丙寅", "甲午"], &[("水", Good), ("火", Bad)], "壬丁", "合去", "bad"),
            // 例二 丁日身弱：壬官為忌，合來 → 凶
            (["丙午", "壬辰", "丁巳", "庚戌"], &[("木", Good), ("火", Good), ("水", Bad)], "壬丁", "合來", "bad"),
            // 例三 丁日身弱忌官：合來忌神增凶
            (["丙申", "壬辰", "丁巳", "辛亥"], &[("木", Good), ("火", Good), ("水", Bad)], "壬丁", "合來", "bad"),
            // 例四 丙日身強喜財：年辛被月丙合去 → 損吉
            (["辛卯", "丙申", "丙午", "乙未"], &[("金", Good), ("水", Good), ("火", Bad)], "辛丙", "合去", "bad"),
            // 例五 甲日身強喜財：合來喜神增吉
            (["戊寅", "己未", "甲寅", "丁卯"], &[("土", Good), ("火", Good), ("木", Bad)], "己甲", "合來", "good"),
            // 例六 甲日身弱忌財：合來忌神增凶
            (["癸酉", "己未", "甲午", "丙子"], &[("水", Good), ("木", Good), ("土", Bad)], "己甲", "合來", "bad"),
        ];
        for (names, entries, stems, direction, verdict) in cases {
            let list = combos(names, entries);
            let item = find(&list, stems, "bound");
            assert_eq!(item.direction.as_deref(), Some(direction), "{names:?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
    }

    #[test]
    fn zheng_he_only_listed_layouts() {
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str); 4] = [
            // 例一 丙日身強喜財：雙丙爭合辛 → 損財
            (["丙辰", "辛卯", "丙寅", "丁酉"], &[("金", Good), ("火", Bad)], "bad"),
            // 例二 甲日身弱忌財：雙甲爭合己 → 幫身分財
            (["甲寅", "己巳", "甲戌", "丙子"], &[("木", Good), ("土", Bad)], "good"),
            // 例三 丁日身弱忌官：雙壬夾丁 → 終非美事
            (["丙子", "壬辰", "丁亥", "壬寅"], &[("木", Good), ("水", Bad)], "bad"),
            // 例四 辛日身旺喜官：三丙爭合辛 → 反而有助
            (["丙申", "丙申", "辛卯", "丙申"], &[("火", Good), ("金", Bad)], "good"),
        ];
        for (names, entries, verdict) in cases {
            let list = combos(names, entries);
            assert_eq!(list.len(), 1, "{names:?} {list:#?}");
            assert_eq!(list[0].kind_label, "爭合妒合");
            assert_eq!(list[0].verdict, verdict, "{names:?} {:#?}", list[0]);
        }
        assert_eq!(combos(["丙申", "丙申", "辛卯", "丙申"], &[])[0].positions.len(), 4);

        // 不在講義排列內（月己、日甲、時甲）不論爭合。
        let list = combos(["丙寅", "己巳", "甲子", "甲戌"], &[]);
        assert!(list.iter().all(|item| item.kind != "contested"), "{list:#?}");
    }

    #[test]
    fn he_er_neng_hua_needs_transforming_seats() {
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, &str); 4] = [
            // 例一 丙日身旺：戊癸臨巳午化火 → 化忌為凶
            (["癸巳", "戊午", "丙午", "壬辰"], &[("水", Good), ("火", Bad)], "癸戊", "bad"),
            // 例二 甲日身弱：甲己臨未戌化土 → 合財化財，凶
            (["己未", "甲戌", "甲子", "戊辰"], &[("水", Good), ("木", Good), ("土", Bad)], "己甲", "bad"),
            // 例三 甲日身弱喜印：丙辛臨申酉化水 → 吉
            (["辛酉", "丙申", "甲寅", "戊辰"], &[("水", Good), ("木", Good), ("金", Bad)], "辛丙", "good"),
            // 例四 丁日身強：戊癸臨巳午化火 → 化忌為凶
            (["癸巳", "戊午", "丁亥", "甲辰"], &[("土", Good), ("火", Bad)], "癸戊", "bad"),
        ];
        for (names, entries, stems, verdict) in cases {
            let list = combos(names, entries);
            let item = find(&list, stems, "transformed");
            assert_eq!(item.scope, "日干以外之合");
            assert!(!item.needs_confirmation, "時支見辰");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }

        // 例五：日時甲己合，土滿盤 → 化氣，類似專旺稼穡。
        let list = combos(["己未", "戊辰", "己未", "甲戌"], &[]);
        assert!(find(&list, "己甲", "transformed").verdict_label.contains("專旺"));
        // 例六：甲日合己，其餘皆火土 → 日主不論化，合成從象。
        let list = combos(["戊戌", "戊午", "甲戌", "己巳"], &[]);
        assert!(find(&list, "甲己", "transformed").verdict_label.contains("從象"));

        // 坐支化神通根乘旺但時支不是辰：10/21 老師列生在辰時為第一條件 → 合而不化。
        let list = combos(["癸巳", "戊午", "丙午", "壬寅"], &[]);
        let item = find(&list, "癸戊", "bound");
        assert!(item.notes.iter().any(|note| note.contains("不是辰")), "{item:#?}");
    }

    fn relation<'a>(list: &'a [LessonStemRelation], id: &str) -> &'a LessonStemRelation {
        list.iter()
            .find(|item| item.id == id)
            .unwrap_or_else(|| panic!("找不到 {id}：{list:#?}"))
    }

    #[test]
    fn stem_generation_and_control_follow_distance_rules() {
        // 通關：甲年丙月戊日，論轉生不論剋。
        let list = classify_relations(&chart(["甲子", "丙寅", "戊辰", "庚申"]));
        let item = relation(&list, "stem-relation-1-3");
        assert_eq!(item.status, "ignored");
        assert!(item.reason.contains("通關"));

        // 貪生忘剋：庚癸甲，不論庚剋甲。
        let list = classify_relations(&chart(["庚申", "癸亥", "甲子", "甲子"]));
        assert_eq!(relation(&list, "stem-relation-1-3").status, "ignored");

        // 先受剋者無力剋人：甲年庚月戊日，不論甲剋戊。
        let list = classify_relations(&chart(["甲子", "庚申", "戊辰", "戊午"]));
        let item = relation(&list, "stem-relation-1-3");
        assert_eq!(item.status, "ignored");
        assert!(item.reason.contains("先受剋"));

        // 近者優先＋連環剋：甲年戊月壬日丙時。
        let list = classify_relations(&chart(["甲寅", "戊辰", "壬子", "丙寅"]));
        let first = relation(&list, "stem-relation-1-2");
        assert_eq!(first.status, "active");
        assert_eq!(first.distance, "鄰近");
        assert!(first.reason.contains("近者優先"), "{}", first.reason);
        let chain = relation(&list, "stem-relation-2-3");
        assert_eq!(chain.status, "active");
        assert!(chain.reason.contains("連環剋"), "{}", chain.reason);

        // 連環剋：戊坐子無根，受甲剋就不剋壬。
        let list = classify_relations(&chart(["甲寅", "戊子", "壬子", "丙寅"]));
        assert_eq!(relation(&list, "stem-relation-2-3").status, "ignored");

        // 土旺木折：甲坐申無根，戊土結黨有根。
        let list = classify_relations(&chart(["甲申", "戊辰", "戊戌", "己丑"]));
        assert_eq!(relation(&list, "stem-relation-1-2").status, "weak");

        // 五合兩干不列入生剋（另見五合）。
        let list = classify_relations(&chart(["甲寅", "己巳", "丙午", "丁酉"]));
        assert!(list.iter().all(|item| item.id != "stem-relation-1-2"));
    }

    #[test]
    fn three_step_analysis_carries_stem_interactions() {
        let result = analyze(&chart(["壬辰", "丁未", "丙寅", "甲午"]), &reference(), &[]);
        assert_eq!(result.stem_interactions.title, "天干生剋合化");
        assert_eq!(result.stem_interactions.combinations[0].stems, "壬丁");
        assert!(result.sources.iter().any(|source| source.date == "2026-10-14"));
    }
}


// ---- 2026-10-21：地支沖合（講義第二十四章第一、二節例題） ----

mod branch_interaction_cases {
    use super::super::branch_interactions::classify_branch_interactions;
    use super::super::stem_interactions::Favor::{self, Bad, Good};
    use super::*;

    fn favor(entries: &'static [(&'static str, Favor)]) -> impl Fn(&str) -> Favor {
        move |element: &str| {
            entries
                .iter()
                .find(|(name, _)| *name == element)
                .map(|(_, favor)| *favor)
                .unwrap_or(Favor::Neutral)
        }
    }

    fn run(
        names: [&str; 4],
        entries: &'static [(&'static str, Favor)],
    ) -> (Vec<LessonBranchClash>, Vec<LessonBranchCombination>) {
        classify_branch_interactions(&chart(names), &favor(entries))
    }

    #[test]
    fn clash_examples_follow_handout_verdicts() {
        // (四柱, 老師喜忌, 沖, 勝方, verdict)
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, Option<&str>, &str); 7] = [
            // 例一 乙弱喜比劫：卯酉沖，酉得令 → 去喜存忌凶
            (["癸酉", "辛酉", "乙卯", "己卯"], &[("木", Good), ("水", Good), ("金", Bad)], "酉卯", Some("月支酉"), "bad"),
            // 例二 壬旺喜財：子午沖，水勝火敗 → 凶
            (["癸亥", "庚申", "壬子", "丙午"], &[("火", Good), ("木", Good), ("水", Bad)], "子午", Some("日支子"), "bad"),
            // 例三 壬弱忌財：巳亥沖，水得令 → 去忌存喜吉
            (["乙巳", "丁亥", "壬午", "庚戌"], &[("水", Good), ("金", Good), ("火", Bad), ("土", Bad)], "巳亥", Some("月支亥"), "good"),
            // 例四 甲強忌劫：卯酉沖，金得令 → 去忌存喜吉
            (["乙卯", "乙酉", "甲寅", "辛未"], &[("金", Good), ("土", Good), ("水", Bad), ("木", Bad)], "卯酉", Some("月支酉"), "good"),
            // 例五 甲旺：三寅沖一申，衰神沖旺旺神發 → 凶
            (["戊寅", "甲寅", "甲寅", "壬申"], &[("火", Good), ("木", Bad), ("金", Bad)], "寅申", Some("日支寅"), "bad"),
            // 例六 丙旺用官：雙午沖子 → 去喜存忌凶
            (["辛巳", "甲午", "丙子", "甲午"], &[("水", Good), ("火", Bad)], "午子", Some("月支午"), "bad"),
            // 例七 戊旺忌印：巳亥沖，火得令、水得金生 → 各有損傷
            (["辛亥", "癸巳", "戊申", "丁巳"], &[("金", Good), ("水", Good), ("火", Bad), ("土", Bad)], "亥巳", Some("月支巳"), "mixed"),
        ];
        for (names, entries, branches, winner, verdict) in cases {
            let (clashes, _) = run(names, entries);
            assert_eq!(clashes.len(), 1, "{names:?} {clashes:#?}");
            let item = &clashes[0];
            assert_eq!(item.branches, branches, "{names:?}");
            assert_eq!(item.winner.as_deref(), winner, "{names:?} {item:#?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
        let (clashes, _) = run(["辛亥", "癸巳", "戊申", "丁巳"], &[]);
        assert_eq!(clashes[0].outcome, "both_damaged");
        let (clashes, _) = run(["戊寅", "甲寅", "甲寅", "壬申"], &[("木", Bad)]);
        assert!(clashes[0].notes.iter().any(|note| note.contains("衰神沖旺")));
    }

    #[test]
    fn storage_clash_touching_month_makes_earth_stronger() {
        // 六合例四：丑未沖在年月，土星獨旺（非子丑合化土）。
        let (clashes, combinations) = run(["己未", "丁丑", "戊子", "己未"], &[]);
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes[0].outcome, "storage");
        assert!(clashes[0].verdict_label.contains("土沖土愈旺"));
        let combo = combinations.iter().find(|item| item.branches == "丑子").unwrap();
        assert_eq!(combo.kind, "controlling");
    }

    #[test]
    fn six_combination_examples_follow_handout_verdicts() {
        // (四柱, 老師喜忌, 合, kind, verdict)
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, &str, &str); 7] = [
            // 例一 丙弱忌金：寅亥合流通金氣轉生木 → 吉
            (["辛亥", "庚寅", "丙子", "乙未"], &[("木", Good), ("火", Good), ("金", Bad)], "亥寅", "generating", "good"),
            // 例二 甲旺忌印喜財：寅亥合水生木剋土 → 凶
            (["丁亥", "壬寅", "甲戌", "甲子"], &[("土", Good), ("水", Bad), ("木", Bad)], "亥寅", "generating", "bad"),
            // 例三 丙弱喜比劫：子丑合絆住水 → 吉
            (["壬子", "癸丑", "丙午", "壬辰"], &[("火", Good), ("木", Good), ("水", Bad)], "子丑", "controlling", "good"),
            // 例五 乙弱：辰酉合土生金，加重沖力 → 凶
            (["庚辰", "乙酉", "乙卯", "甲申"], &[("木", Good), ("水", Good), ("金", Bad), ("土", Bad)], "辰酉", "generating", "bad"),
            // 例六 甲弱：巳申合火剋金，減低沖力（亡羊補牢）
            (["丁巳", "戊申", "甲寅", "癸酉"], &[("水", Good), ("木", Good), ("金", Bad), ("火", Bad), ("土", Bad)], "巳申", "controlling", "good"),
            // 例七 丙弱財旺：卯戌合木剋土存火 → 幫身分財吉
            (["庚申", "乙酉", "丙戌", "丁卯"], &[("木", Good), ("火", Good), ("金", Bad), ("土", Bad)], "戌卯", "controlling", "good"),
            // 例八 癸弱：午未合火生土逼水 → 凶
            (["己未", "庚午", "癸亥", "己未"], &[("金", Good), ("水", Good), ("火", Bad), ("土", Bad)], "未午", "generating", "bad"),
        ];
        for (names, entries, branches, kind, verdict) in cases {
            let (_, combinations) = run(names, entries);
            let item = combinations
                .iter()
                .find(|item| item.branches == branches)
                .unwrap_or_else(|| panic!("{names:?} {combinations:#?}"));
            assert_eq!(item.kind, kind, "{names:?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }

        let (_, combinations) = run(["庚辰", "乙酉", "乙卯", "甲申"], &[]);
        let effect = combinations[0].clash_effect.as_deref().unwrap_or_default();
        assert!(effect.contains("加重沖力"), "{effect}");
        let (_, combinations) = run(["丁巳", "戊申", "甲寅", "癸酉"], &[]);
        let effect = combinations[0].clash_effect.as_deref().unwrap_or_default();
        assert!(effect.contains("減低"), "{effect}");
        let (_, combinations) = run(["壬子", "癸丑", "丙午", "壬辰"], &[]);
        let effect = combinations[0].clash_effect.as_deref().unwrap_or_default();
        assert!(effect.contains("減低"), "{effect}");
        // 寅為喜逢申沖、亥合寅：申貪生亥，減低沖力。
        let (_, combinations) = run(["壬申", "辛亥", "甲寅", "丙申"], &[]);
        let item = combinations.iter().find(|item| item.branches == "亥寅").unwrap();
        assert!(item.clash_effect.as_deref().unwrap_or_default().contains("貪生"), "{item:#?}");
    }

    #[test]
    fn three_step_analysis_carries_branch_interactions() {
        let result = analyze(&chart(["癸酉", "辛酉", "乙卯", "己卯"]), &reference(), &[]);
        assert_eq!(result.branch_interactions.title, "地支沖合會刑害");
        assert_eq!(result.branch_interactions.clashes.len(), 1);
        assert!(result.sources.iter().any(|source| source.date == "2026-10-21"));
    }
}


// ---- 中級二講義第 24 章第三～五節、第 25 章、第 26 章（課堂未講，依講義例題） ----

mod textbook_24_26_cases {
    use super::super::branch_gatherings::{classify_gatherings, classify_punishments};
    use super::super::pillar_relations::classify_pillar_relations;
    use super::super::stem_interactions::Favor::{self, Bad, Good};
    use super::*;

    fn favor(entries: &'static [(&'static str, Favor)]) -> impl Fn(&str) -> Favor {
        move |element: &str| {
            entries
                .iter()
                .find(|(name, _)| *name == element)
                .map(|(_, favor)| *favor)
                .unwrap_or(Favor::Neutral)
        }
    }

    fn gatherings(names: [&str; 4], entries: &'static [(&'static str, Favor)]) -> Vec<LessonBranchGathering> {
        classify_gatherings(&chart(names), &favor(entries))
    }

    fn find<'a>(list: &'a [LessonBranchGathering], branches: &str) -> &'a LessonBranchGathering {
        list.iter()
            .find(|item| item.branches == branches)
            .unwrap_or_else(|| panic!("{branches} not in {list:#?}"))
    }

    #[test]
    fn three_combinations_follow_handout_examples() {
        // (四柱, 講義喜忌, 局, 成局, verdict)
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, bool, &str); 6] = [
            // 例一 戊弱：申子辰水局、時透壬 → 三合不吉
            (["庚申", "戊子", "戊辰", "壬戌"], &[("火", Good), ("土", Good), ("水", Bad)], "申子辰", true, "bad"),
            // 例二 乙旺：亥卯未木局 → 徒添身旺不吉
            (["甲寅", "丁卯", "乙未", "丁亥"], &[("火", Good), ("木", Bad)], "亥卯未", true, "bad"),
            // 例三 炎上：寅午戌火局、月透丙 → 助格為吉
            (["甲戌", "丙寅", "丙午", "庚寅"], &[("火", Good), ("木", Good), ("金", Bad)], "寅午戌", true, "good"),
            // 例四、例五：旺神不當令，以常理論
            (["壬午", "乙巳", "乙丑", "乙酉"], &[("水", Good), ("木", Good)], "巳酉丑", false, "neutral"),
            (["甲子", "戊辰", "甲申", "庚午"], &[("水", Good), ("木", Good)], "申子辰", false, "neutral"),
            // 26 章地支例十九：寅午戌三合火局，月透丙（戌仍是戌）
            (["庚午", "丙戌", "甲寅", "丙寅"], &[("金", Good), ("水", Good), ("木", Bad)], "寅午戌", true, "neutral"),
        ];
        for (names, entries, branches, formed, verdict) in cases {
            let list = gatherings(names, entries);
            let item = find(&list, branches);
            assert_eq!(item.formed, formed, "{names:?} {item:#?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
        // 例四、例五：整組依常理論，不再拆半合。
        assert!(gatherings(["壬午", "乙巳", "乙丑", "乙酉"], &[]).iter().all(|item| item.kind != "half_combination"));
        assert!(gatherings(["甲子", "戊辰", "甲申", "庚午"], &[]).iter().all(|item| item.kind != "half_combination"));
        // 時干透局神 → 合力次之；合神居月令且月干透 → 最大。
        assert!(find(&gatherings(["庚申", "戊子", "戊辰", "壬戌"], &[]), "申子辰").force_label.contains("次之"));
        assert!(find(&gatherings(["甲寅", "乙卯", "乙未", "丁亥"], &[]), "亥卯未").force_label.contains("最大"));
    }

    #[test]
    fn half_combinations_follow_handout_examples() {
        // 第三節例六 甲旺：卯未半合木剋土，此半合無情不吉
        let list = gatherings(["丁未", "癸卯", "甲辰", "乙亥"], &[("火", Good), ("土", Good), ("木", Bad)]);
        assert!(!find(&list, "亥卯未").formed, "辰隔開亥卯未");
        assert_eq!(find(&list, "未卯").verdict, "bad");
        // 第三節例七 甲弱：申子半合水局，印能化殺
        let list = gatherings(["己酉", "甲戌", "甲申", "丙子"], &[("水", Good), ("木", Good), ("金", Bad)]);
        assert_eq!(find(&list, "申子").verdict, "good");
        // 第三節例八 丙旺：巳酉半合（巳居月令 → 火剋金）、寅午半合，木生火旺剋金不吉
        let list = gatherings(["辛酉", "癸巳", "丙午", "庚寅"], &[("土", Good), ("金", Good), ("木", Bad), ("火", Bad)]);
        let si_you = find(&list, "酉巳");
        assert!(!si_you.formed && si_you.kind_label.contains("依本氣生剋"), "{si_you:#?}");
        assert_eq!(si_you.verdict, "bad");
        assert_eq!(find(&list, "午寅").verdict, "bad");
        // 26 章地支例八：亥卯未被日柱巳火隔開，只論卯未半合剋土存木
        let list = gatherings(["乙未", "己卯", "己巳", "乙亥"], &[("火", Good), ("土", Good), ("木", Bad), ("水", Bad)]);
        assert!(!find(&list, "亥卯未").formed);
        assert_eq!(find(&list, "未卯").verdict, "bad");
        // 講義第 30 頁：申辰缺中神，在命以常理論，須歲運見子填實（自擬四柱）
        let list = gatherings(["庚午", "甲申", "丙辰", "己丑"], &[]);
        assert!(list.iter().any(|item| item.kind == "incomplete_half" && item.branches == "申辰"), "{list:#?}");
    }

    #[test]
    fn three_meetings_follow_handout_examples() {
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, bool, &str); 7] = [
            // 例一 甲旺：寅卯辰會木，忌水木
            (["辛卯", "庚寅", "甲辰", "乙亥"], &[("土", Good), ("金", Good), ("水", Bad), ("木", Bad)], "寅卯辰", true, "bad"),
            // 例二 調候：巳午未會火，忌火土
            (["丙午", "癸巳", "甲午", "辛未"], &[("金", Good), ("水", Good), ("火", Bad), ("土", Bad)], "巳午未", true, "bad"),
            // 例三 丙弱：申酉戌會金，財旺身弱不吉
            (["庚戌", "乙酉", "丙申", "甲午"], &[("火", Good), ("金", Bad)], "申酉戌", true, "bad"),
            // 例四：亥子丑會水，徒增水寒
            (["甲子", "乙亥", "癸丑", "乙卯"], &[("火", Good), ("水", Bad)], "亥子丑", true, "bad"),
            // 例五 乙：申酉戌會金（申居月令），土生金剋木不吉
            (["甲寅", "壬申", "乙酉", "丙戌"], &[("水", Good), ("木", Good), ("金", Bad)], "申酉戌", true, "bad"),
            // 26 章地支例三：寅卯辰三字連珠含月令，三會木方增強木力剋土
            (["辛巳", "壬辰", "辛卯", "庚寅"], &[("土", Good), ("金", Good), ("水", Bad), ("木", Bad)], "寅卯辰", true, "bad"),
            // 26 章地支例十三：申酉戌會神不居月令，只論土旺生金
            (["己酉", "甲戌", "壬申", "辛亥"], &[("木", Good), ("火", Good), ("金", Bad), ("水", Bad)], "申酉戌", false, "neutral"),
        ];
        for (names, entries, branches, formed, verdict) in cases {
            let list = gatherings(names, entries);
            let item = list
                .iter()
                .find(|item| item.branches == branches && item.kind == "three_meeting")
                .unwrap_or_else(|| panic!("{names:?} {list:#?}"));
            assert_eq!(item.formed, formed, "{names:?} {item:#?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
        // 例六：辰戌丑未全，可論土方土局（稼穡喜土）
        let list = gatherings(["戊戌", "丙辰", "己未", "乙丑"], &[("土", Good), ("金", Good)]);
        let earth = list.iter().find(|item| item.kind == "earth_full").unwrap();
        assert_eq!(earth.verdict, "good");
    }

    #[test]
    fn punishments_and_harms_are_read_by_five_elements() {
        let run = |names: [&str; 4], entries: &'static [(&'static str, Favor)]| {
            classify_punishments(&chart(names), &favor(entries))
        };
        // (四柱, 喜忌, kind, 生剋讀法, verdict)
        let cases: [([&str; 4], &'static [(&'static str, Favor)], &str, &str, &str); 7] = [
            // 例一 丑戌未刑＝土厚埋金
            (["癸未", "壬戌", "庚戌", "丁丑"], &[("金", Good), ("土", Bad), ("木", Bad)], "punishment", "同屬土", "bad"),
            // 例二 寅巳申刑＝木生火剋金（辛弱）
            (["丁酉", "壬寅", "辛巳", "丙申"], &[("土", Good), ("金", Good), ("火", Bad)], "punishment", "木生火、火剋金", "bad"),
            // 例三 子卯刑＝水生木
            (["甲戌", "丙子", "癸卯", "壬戌"], &[], "punishment", "水生木", "neutral"),
            // 例四 丑午害＝火生土
            (["甲子", "丁丑", "壬午", "甲辰"], &[("金", Good), ("水", Good), ("火", Bad)], "harm", "火生土", "neutral"),
            // 例六 申亥害＝金生水（丁弱忌金水）
            (["辛未", "丙申", "丁亥", "壬寅"], &[("木", Good), ("火", Good), ("金", Bad), ("水", Bad)], "harm", "金生水", "bad"),
            // 例七 酉酉自刑＝加強金
            (["丙子", "丁酉", "丁酉", "壬寅"], &[("木", Good), ("火", Good), ("金", Bad), ("水", Bad)], "self_punishment", "同屬金", "bad"),
            // 例八 亥亥自刑＝加強水
            (["甲午", "乙亥", "乙亥", "丙戌"], &[("火", Good), ("土", Good), ("水", Bad), ("木", Bad)], "self_punishment", "同屬水", "bad"),
        ];
        for (names, entries, kind, relation, verdict) in cases {
            let list = run(names, entries);
            let item = list
                .iter()
                .find(|item| item.kind == kind)
                .unwrap_or_else(|| panic!("{names:?} {list:#?}"));
            assert!(item.relation.contains(relation), "{names:?} {item:#?}");
            assert_eq!(item.verdict, verdict, "{names:?} {item:#?}");
        }
        // 例五 辰卯害＝木剋土
        let list = run(["庚辰", "己卯", "壬辰", "庚子"], &[]);
        assert!(list.iter().any(|item| item.kind == "harm" && item.relation == "木剋土"), "{list:#?}");
        // 寅申、丑未只兩字時已是相沖，不重複列刑。
        assert!(run(["甲寅", "壬申", "乙酉", "丙戌"], &[]).iter().all(|item| item.kind != "punishment"));
    }

    #[test]
    fn same_pillar_relations_cover_sixty_cycles() {
        let mut counts = std::collections::BTreeMap::new();
        let mut hidden = 0;
        let mut pure = 0;
        for index in 0..60 {
            let cycle = SixtyCycle::from_index(index);
            let cycles = [cycle.clone(), cycle.clone(), cycle.clone(), cycle];
            let item = classify_pillar_relations(&cycles, &|_: &str| Favor::Neutral).remove(0);
            *counts.entry(item.kind.clone()).or_insert(0) += 1;
            if item.hidden_combination.is_some() {
                hidden += 1;
            }
            if item.hidden_combination_pure {
                pure += 1;
            }
        }
        // 講義：五類各十二組；干支相合十二組，其中六組較純粹。
        assert_eq!(counts.values().copied().collect::<Vec<_>>(), vec![12, 12, 12, 12, 12], "{counts:?}");
        assert_eq!((hidden, pure), (12, 6));

        let kinds = classify_pillar_relations(&chart(["甲午", "甲子", "丙申", "戊寅"]), &|_: &str| Favor::Neutral)
            .into_iter()
            .map(|item| item.kind)
            .collect::<Vec<_>>();
        assert_eq!(kinds, ["stem_generates_branch", "branch_generates_stem", "stem_controls_branch", "branch_controls_stem"]);

        // 26 章地支例二十：年柱壬午干支上下相合，絆住午中丁火（丁為喜用）。
        let relations = classify_pillar_relations(
            &chart(["壬午", "己酉", "辛卯", "丁酉"]),
            &favor(&[("火", Good), ("木", Good), ("土", Bad), ("金", Bad)]),
        );
        assert_eq!(relations[0].verdict, "bad", "{:#?}", relations[0]);
        assert!(relations[0].hidden_combination_pure);
        // 喜神坐剋受損：甲為喜、坐申。
        let relations = classify_pillar_relations(&chart(["甲申", "丙子", "戊午", "庚申"]), &favor(&[("木", Good)]));
        assert_eq!(relations[0].verdict, "bad");
    }

    #[test]
    fn interaction_strength_review_flags_handout_reversals() {
        // (四柱, 原課堂判定, 複核判定)：講義都判身弱／身強，與複核一致。
        let cases: [([&str; 4], &str, &str); 3] = [
            // 26 章天干例三：丁壬生辰時合化木 → 戊土財格身弱
            (["丁亥", "壬子", "戊辰", "丙辰"], "強候選", "弱候選"),
            // 26 章地支例三：寅卯辰三會木方 → 辛金印格身弱
            (["辛巳", "壬辰", "辛卯", "庚寅"], "強候選", "弱候選"),
            // 24 章刑害例五：子辰半合水局 → 壬水轉強
            (["庚辰", "己卯", "壬辰", "庚子"], "弱候選", "強候選"),
        ];
        for (names, before, after) in cases {
            let result = analyze(&chart(names), &reference(), &[]);
            assert_eq!(result.strength.classification, before, "{names:?}");
            assert_eq!(result.interaction_strength.classification, after, "{names:?} {:#?}", result.interaction_strength);
            assert!(result.interaction_strength.changed);
        }
        // 沒有合化、成局 → 不變。
        let result = analyze(&chart(["庚寅", "丙戌", "己丑", "乙亥"]), &reference(), &[]);
        assert!(!result.interaction_strength.changed);
        assert!(result.interaction_strength.changes.is_empty(), "{:#?}", result.interaction_strength);
    }
}

#[test]
fn lesson_1104_month_knocked_uses_main_qi_and_mother_overload_is_weak() {
    // 講義第 24 章例一：癸未 壬戌 庚戌 丁丑，戌未刑在月令，時干透丁也不取正官格，取本氣戊＝偏印格；
    // 土厚埋金（母慈滅子），不論身強，只宜金運。
    let result = analyze(&chart(["癸未", "壬戌", "庚戌", "丁丑"]), &reference(), &[]);
    assert_eq!(result.pattern.ordinary_result.as_deref(), Some("偏印格"));
    assert!(result.pattern.conclusion.contains("只剩本氣"), "{}", result.pattern.conclusion);
    assert!(result.pattern.conclusion.contains("丁"), "{}", result.pattern.conclusion);
    assert_eq!(result.strength.classification, "弱候選");
    assert!(!result.strength.root_knockouts.is_empty());
    assert!(result.strength.mother_overload.is_some());
    assert_eq!(result.preferences.plans.len(), 1);
    let plan = &result.preferences.plans[0];
    assert_eq!(plan.rule_id, "weak-seal-overload-use-peer");
    assert_eq!(plan.use_directions[0].element, "金");
    assert!(plan.unfavorable_directions.iter().any(|item| item.element == "土"));
}

#[test]
fn lesson_1104_unknocked_month_keeps_transparent_middle_qi() {
    // 月令戌沒有被相鄰地支刑沖時，仍照透干取格（時干透丁 → 正官格候選）。
    let result = analyze(&chart(["癸卯", "壬戌", "庚子", "丁亥"]), &reference(), &[]);
    assert!(result
        .pattern
        .ordinary_candidates
        .iter()
        .any(|item| item.pattern == "正官格"));
    assert!(result.strength.mother_overload.is_none());
}

#[test]
fn lesson_1104_balanced_chart_gets_balance_hint_and_strong_seal_chart_does_not() {
    // 例三：甲戌 丙子 癸卯 壬戌，講義判中和「行五運皆可，只忌歲運逢沖」。
    let balanced = analyze(&chart(["甲戌", "丙子", "癸卯", "壬戌"]), &reference(), &[]);
    assert!(balanced.strength.balance_hint.is_some());
    // 例八：甲午 乙亥 乙亥 丙戌，印格身旺，有比劫 → 不是母慈滅子，也不算中和。
    let strong = analyze(&chart(["甲午", "乙亥", "乙亥", "丙戌"]), &reference(), &[]);
    assert!(strong.strength.mother_overload.is_none());
    assert!(matches!(strong.strength.classification.as_str(), "強候選" | "旺候選"));
}
