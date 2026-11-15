use crate::bazi::{
    DaYunDto, LiuNianDto, QuantAnnualLuckScore, QuantCongPattern, QuantLessonAppReference,
    QuantLessonCandidate, QuantLessonContent, QuantLessonInference, QuantLuckScore,
    QuantModelInteraction, QuantModelPillarScore, QuantModelResponse, QuantModelSubScore,
    QuantShaYin, QuantSpecialPattern, QuantYongShen,
};
use crate::lesson_analysis::{analyze as analyze_lesson, LessonModelReference};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use tyme4rs::tyme::sixtycycle::{EarthBranch, HeavenStem, SixtyCycle};
use tyme4rs::tyme::Culture;

mod lesson_0902;

#[derive(Debug, Clone)]
struct CalculationInputs {
    stems: Vec<StemInput>,
    branches: Vec<BranchInput>,
}

#[derive(Debug, Clone)]
struct StemInput {
    pillar: String,
    stem: String,
    position_index: usize,
}

#[derive(Debug, Clone)]
struct BranchInput {
    pillar: String,
    branch: String,
    hidden_stems: Vec<String>,
    position_index: usize,
}

#[derive(Debug, Clone)]
struct HalfCombinationCandidate {
    left_index: usize,
    right_index: usize,
    element: String,
    distance: usize,
    transforms: bool,
    note: String,
    combine_note: String,
}

impl HalfCombinationCandidate {
    fn involves(&self, position_index: usize) -> bool {
        self.left_index == position_index || self.right_index == position_index
    }
}

#[derive(Debug, Clone)]
struct ThreeCombinationCandidate {
    first_index: usize,
    second_index: usize,
    third_index: usize,
    element: String,
    transforms: bool,
    label: String,
}

impl ThreeCombinationCandidate {
    fn involves(&self, position_index: usize) -> bool {
        self.first_index == position_index
            || self.second_index == position_index
            || self.third_index == position_index
    }
}

#[derive(Debug, Clone)]
struct ThreeMeetingCandidate {
    first_index: usize,
    second_index: usize,
    third_index: usize,
    element: String,
    transforms: bool,
    label: String,
}

impl ThreeMeetingCandidate {
    fn involves(&self, position_index: usize) -> bool {
        self.first_index == position_index
            || self.second_index == position_index
            || self.third_index == position_index
    }
}

#[derive(Debug, Clone)]
struct BranchPruneEffect {
    remaining_stem: String,
    remaining_ratio: f64,
    note: String,
}

#[derive(Debug, Clone)]
struct SixCombinationEffect {
    other_index: usize,
    pair: String,
    element: String,
    remainder: f64,
}

#[derive(Debug, Clone)]
struct StemCombinationEffect {
    pair: String,
    target_element: String,
    remainder: f64,
    transforms: bool,
    note: String,
    combine_note: String,
}

#[derive(Debug, Clone)]
struct StemClashEffect {
    remainder: f64,
    note: String,
}

#[derive(Debug, Clone)]
struct BranchQiModifier {
    stem_multipliers: HashMap<String, f64>,
    element_multipliers: HashMap<String, f64>,
    note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BranchTransformCategory {
    SixCombination,
    HalfCombination,
    ThreeCombination,
    ThreeMeeting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombinationBranchRole {
    Specialist,
    Growth,
    Storage,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchTransformSliceKind {
    Primary,
    Secondary,
    Residual,
}

#[derive(Debug, Clone)]
struct BranchTransformSlice {
    kind: BranchTransformSliceKind,
    display_stem: String,
    ratio: f64,
    transformed: bool,
    hidden_stem_index: isize,
    interaction: String,
    factor: f64,
    transparency_base_factor: f64,
    root_factor: f64,
    note: String,
}

#[derive(Debug, Clone)]
struct BranchTransform {
    category: BranchTransformCategory,
    role: CombinationBranchRole,
    element: String,
    note: String,
    combine_note: String,
    factor: f64,
    residual_factor: f64,
    slices: Vec<BranchTransformSlice>,
}

#[derive(Debug, Clone)]
struct ResolvedChartState {
    stems: Vec<StemInput>,
    branches: Vec<BranchInput>,
    three_meeting_candidates: Vec<ThreeMeetingCandidate>,
    three_combination_candidates: Vec<ThreeCombinationCandidate>,
    half_combination_candidates: Vec<HalfCombinationCandidate>,
    branch_transforms: HashMap<usize, BranchTransform>,
    six_combination_effects: HashMap<usize, SixCombinationEffect>,
    branch_prune_effects: HashMap<usize, BranchPruneEffect>,
    branch_qi_modifiers: HashMap<usize, BranchQiModifier>,
    stem_combination_effects: HashMap<usize, StemCombinationEffect>,
    stem_clash_effects: HashMap<usize, StemClashEffect>,
    six_clash_resolution: SixCombinationClashResolution,
    non_transform_boosts: HashMap<usize, Vec<NonTransformBoost>>,
}

/// 三合、三會不化時，相關五行專氣的提升量（以「個五行專氣」為單位）。
#[derive(Debug, Clone)]
struct NonTransformBoost {
    element: String,
    extra_units: f64,
    note: String,
}

#[derive(Debug, Clone)]
struct ResolvedScoreBundle {
    resolved_state: ResolvedChartState,
    stem_scores: Vec<QuantModelPillarScore>,
    branch_scores: Vec<QuantModelPillarScore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootLevel {
    None,
    Minor,
    Middle,
    Tomb,
    Lu,
}

impl RootLevel {
    fn label(self) -> &'static str {
        match self {
            RootLevel::None => "無根",
            RootLevel::Minor => "微根",
            RootLevel::Middle => "中根",
            RootLevel::Tomb => "墓根",
            RootLevel::Lu => "祿根",
        }
    }

    fn rank(self) -> i32 {
        match self {
            RootLevel::None => 0,
            RootLevel::Minor => 1,
            RootLevel::Middle => 2,
            RootLevel::Tomb => 3,
            RootLevel::Lu => 4,
        }
    }
}

#[derive(Debug, Clone)]
struct RootApplication {
    level: RootLevel,
    support_stem: String,
    reason: String,
    branch: String,
    qi_multiplier: f64,
}

impl RootApplication {
    fn none() -> Self {
        Self {
            level: RootLevel::None,
            support_stem: String::new(),
            reason: String::new(),
            branch: String::new(),
            qi_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
struct TransparencyApplication {
    stem_pillar: String,
    visible_stem: String,
    element: String,
    multiplier: f64,
    bonus: f64,
    kind: String,
}

impl TransparencyApplication {
    fn label(&self) -> String {
        format!("{}{}@{}", self.kind, self.visible_stem, self.visible_stem)
    }
}

#[derive(Debug, Clone)]
struct BranchQiSlice {
    display_stem: String,
    ten_god: String,
    ratio: f64,
    row_raw_score: f64,
    adjusted_raw_contribution: f64,
    position_adjusted: f64,
    base_interaction: String,
    base_note: String,
    transformed: bool,
    hidden_stem_index: isize,
    transparency_key: String,
    transparency_base: f64,
    transform_slice_kind: BranchTransformSliceKind,
}

#[derive(Debug, Clone)]
struct BranchInteraction {
    multiplier: f64,
    note: String,
    combine_note: String,
    priority: i32,
}

impl BranchInteraction {
    fn none() -> Self {
        Self {
            multiplier: 1.0,
            note: String::new(),
            combine_note: String::new(),
            priority: 0,
        }
    }

    fn clash(note: String, multiplier: f64, other: &BranchInput) -> Self {
        Self {
            multiplier,
            note,
            combine_note: format!("沖@{}{}", other.pillar, other.branch),
            priority: 3,
        }
    }

    fn combine(note: String, multiplier: f64, other: &BranchInput) -> Self {
        Self {
            multiplier,
            note,
            combine_note: format!("合@{}{}", other.pillar, other.branch),
            priority: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchClashType {
    Specialist,
    Growth,
    Storage,
}

pub(crate) fn build_quant_model_response(
    day_master: HeavenStem,
    natal_cycles: &[SixtyCycle; 4],
    da_yun: &[DaYunDto],
    normal_pattern_reference: &str,
) -> QuantModelResponse {
    let day_master_name = stem_name(day_master.clone());
    let inputs = build_natal_inputs(natal_cycles);
    let bundle = score_resolved_chart(&day_master_name, &inputs);
    let stem_total = round1(bundle.stem_scores.iter().map(|item| item.final_score).sum());
    let branch_total = round1(
        bundle
            .branch_scores
            .iter()
            .map(|item| item.final_score)
            .sum(),
    );
    let total = round1(stem_total + branch_total);
    let strength_label = if total >= 0.0 {
        "偏強".to_string()
    } else {
        "偏弱".to_string()
    };
    let summary = format!(
        "量化五行力（Phase 2.1）總分 {:.1}，判定為{}。",
        total, strength_label
    );
    let note = "已改為較貼近《量化五行力》的逐項計分：先沖合耗損，再依表計算得根與透干；根氣與透干一律以五行為準，不再分陰陽。仍屬工程化重建版，未完整覆蓋全書所有特例。".to_string();
    let yong_shen = Some(analyze_yong_shen(
        &day_master_name,
        total,
        &bundle.stem_scores,
        &bundle.branch_scores,
    ));
    let cong_pattern = Some(analyze_cong_pattern(
        total,
        &strength_label,
        &bundle.stem_scores,
        &bundle.branch_scores,
    ));
    let special_pattern = Some(analyze_special_pattern(
        normal_pattern_reference,
        total,
        &bundle,
        cong_pattern.as_ref().expect("cong pattern should exist"),
    ));
    let sha_yin = Some(analyze_sha_yin(
        &day_master_name,
        total,
        &bundle.stem_scores,
        &bundle.branch_scores,
    ));
    let interactions = build_natal_interactions(&bundle.resolved_state, &bundle.branch_scores);
    let lesson_content = Some(build_lesson_content(
        natal_cycles,
        &day_master_name,
        normal_pattern_reference,
        &strength_label,
        yong_shen.as_ref().expect("yong shen analysis should exist"),
        special_pattern
            .as_ref()
            .expect("special pattern analysis should exist"),
        &interactions,
    ));
    let luck_scores = Some(build_luck_scores(&day_master_name, total, &inputs, da_yun));

    QuantModelResponse {
        day_master: day_master_name,
        summary,
        note,
        stem_score_total: stem_total,
        branch_score_total: branch_total,
        total_score: total,
        strength_label,
        yong_shen,
        cong_pattern,
        special_pattern,
        sha_yin,
        lesson_content,
        lu_strength: Some(crate::lu_strength::analyze(natal_cycles)),
        interactions: Some(interactions),
        stem_scores: bundle.stem_scores,
        branch_scores: bundle.branch_scores,
        luck_scores,
    }
}

fn build_lesson_content(
    natal_cycles: &[SixtyCycle; 4],
    day_master: &str,
    normal_pattern_reference: &str,
    strength_label: &str,
    yong_shen: &QuantYongShen,
    special_pattern: &QuantSpecialPattern,
    interactions: &[QuantModelInteraction],
) -> QuantLessonContent {
    let three_step_analysis = Some(analyze_lesson(
        natal_cycles,
        &LessonModelReference {
            ge_ju: normal_pattern_reference.to_string(),
            strength_label: strength_label.to_string(),
            use_elements: yong_shen.favorable_elements.clone(),
            special_pattern: special_pattern.primary_pattern.clone(),
            special_category: special_pattern
                .primary_pattern
                .as_ref()
                .map(|_| special_pattern.category.clone()),
        },
        interactions,
    ));
    QuantLessonContent {
        title: "上課內容".to_string(),
        status: "課堂推論已整理（來源含 2026/07/01、2026/07/08、2026/07/15、2026/07/22、2026/07/29、2026/08/05、2026/08/12、2026/08/19、2026/08/26、2026/09/02；未核對規則不視為已驗證）".to_string(),
        analysis_order: vec![
            "格局判定".to_string(),
            "日主強弱".to_string(),
            "用神與喜忌".to_string(),
        ],
        future_rule_notice: "完整課堂規則與來源保存在 bazi-skills 文件；已接入規則以事實、規則與證據鏈顯示，未核對內容不納入自動推論。"
            .to_string(),
        inferences: build_lesson_inferences(
            natal_cycles,
            day_master,
            normal_pattern_reference,
            strength_label,
            yong_shen,
            special_pattern,
            interactions,
        ),
        three_step_analysis,
    }
}

fn build_lesson_inferences(
    natal_cycles: &[SixtyCycle; 4],
    day_master: &str,
    normal_pattern_reference: &str,
    strength_label: &str,
    yong_shen: &QuantYongShen,
    special_pattern: &QuantSpecialPattern,
    interactions: &[QuantModelInteraction],
) -> Vec<QuantLessonInference> {
    let chart = natal_cycles
        .iter()
        .map(|cycle| {
            format!(
                "{}{}",
                stem_name(cycle.get_heaven_stem()),
                branch_name(cycle.get_earth_branch())
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let app_reference = QuantLessonAppReference {
        ge_ju: normal_pattern_reference.to_string(),
        strength_label: strength_label.to_string(),
        special_pattern: special_pattern
            .primary_pattern
            .clone()
            .unwrap_or_else(|| special_pattern.category.clone()),
    };

    let two_element_triggered = special_pattern.category == "兩神成象";
    let mut inferences = vec![
        build_20260812_xin_wealth_case_inference(&chart, &app_reference, interactions),
        build_20260812_disease_medicine_case_inference(&chart, &app_reference),
        build_20260812_follow_wealth_case_inference(&chart, &app_reference, interactions),
        build_20260805_fu_yi_priority_inference(&app_reference, yong_shen),
        build_20260805_bing_fire_case_inference(&chart, &app_reference),
        build_20260722_day_master_state_inference(natal_cycles, day_master, &app_reference),
        build_20260729_geng_shen_case_inference(&chart, &app_reference, interactions),
        QuantLessonInference {
            rule_id: "20260715-two-element-normalization".to_string(),
            title: "兩神成像格名簡化".to_string(),
            status: if two_element_triggered {
                "已觸發・保留正格"
            } else {
                "未觸發"
            }
            .to_string(),
            source_verification: "lesson_transcript_summary".to_string(),
            algorithm_status: "implemented".to_string(),
            match_status: if two_element_triggered {
                "matched"
            } else {
                "not_matched"
            }
            .to_string(),
            output_mode: "annotation_only".to_string(),
            fixture_scope: "rule_validation".to_string(),
            agreement_status: "not_comparable".to_string(),
            fallback_policy: "preserve_normal_outputs".to_string(),
            app_reference: app_reference.clone(),
            lesson_candidate: if two_element_triggered {
                Some(QuantLessonCandidate {
                    pattern: special_pattern
                        .primary_pattern
                        .clone()
                        .unwrap_or_else(|| "兩神成像候選".to_string()),
                    strength: "不另判".to_string(),
                    use_elements: special_pattern.special_use_gods.clone(),
                })
            } else {
                None
            },
            sensitive_claims_policy: "excluded".to_string(),
            conclusion: if two_element_triggered {
                format!(
                "量化模型偵測到{}；依 2026/07/15 本課派規則，它只作兩行集中描述，仍保留普通格「{}」作主要比較，不直接覆蓋。",
                special_pattern
                    .primary_pattern
                    .as_deref()
                    .unwrap_or("兩神成像候選"),
                normal_pattern_reference
            )
            } else {
                format!(
                    "本命未觸發兩神成像候選；目前仍以普通格「{}」及既有特殊格檢查結果為準。",
                    normal_pattern_reference
                )
            },
            evidence: vec![
                format!("本命四柱：{}。", chart),
                format!("普通格參照：{}。", normal_pattern_reference),
                format!("特殊格檢查：{}。", special_pattern.conclusion),
            ],
            rejected_alternatives: vec![
                "不因兩種五行集中就自動覆蓋普通格。".to_string(),
                "不使用四柱中不存在的暗沖、暗合來補造財官。".to_string(),
            ],
        },
    ];

    inferences.extend(lesson_0902::annotations(&chart, &app_reference, interactions));

    let exact_case_matched = chart == "丁巳 己酉 己卯 乙丑";
    inferences.push(QuantLessonInference {
        rule_id: "20260715-case-1977".to_string(),
        title: "2026/07/15 課堂命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_and_whiteboard_confirmed".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: if !exact_case_matched || normal_pattern_reference == "直輸四柱分析" {
            "not_comparable"
        } else if normal_pattern_reference == "食神格" && strength_label == "偏弱" {
            "agree"
        } else {
            "conflict"
        }
        .to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference,
        lesson_candidate: if exact_case_matched {
            Some(QuantLessonCandidate {
                pattern: "食神格".to_string(),
                strength: "身弱".to_string(),
                use_elements: vec!["火".to_string()],
            })
        } else {
            None
        },
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "App 普通格為「{}」、量化強弱為「{}」；課堂候選值為食神格身弱、以火扶身。兩者並列作證據比較，課堂值不覆蓋 App 量化結果。",
                normal_pattern_reference, strength_label
            )
        } else {
            format!(
                "本命四柱「{}」不是 2026/07/15 課例「丁巳 己酉 己卯 乙丑」，不套用食神格身弱、火扶身的課堂候選值。",
                chart
            )
        },
        evidence: if exact_case_matched {
            vec![
                "己土生於酉月，酉藏辛；辛對己土為食神，因此課堂普通格取食神格。".to_string(),
                "月支酉與日支卯形成卯酉沖；時干乙七殺在日支卯得根。".to_string(),
                "年干丁與年支巳提供火印扶身來源；火是否成 App 正式用神仍與量化結果並列判讀。".to_string(),
                "大運由戊申起逆排，45 歲進甲辰、55 歲進癸卯；只作相對運勢比較。".to_string(),
            ]
        } else {
            vec![
                format!("目前四柱：{}。", chart),
                "精確命例 predicate：丁巳／己酉／己卯／乙丑，任一柱不同即不套用。".to_string(),
            ]
        },
        rejected_alternatives: vec![
            "不覆蓋 App 的普通格與量化強弱。".to_string(),
            "敏感人生事件敘述不納入自動推論。".to_string(),
        ],
    });

    inferences
}

fn build_20260812_xin_wealth_case_inference(
    chart: &str,
    app_reference: &QuantLessonAppReference,
    interactions: &[QuantModelInteraction],
) -> QuantLessonInference {
    let exact_case_matched = chart == "癸卯 乙卯 辛亥 庚寅";
    let mut evidence = vec![format!("目前四柱：{}。", chart)];
    if exact_case_matched {
        evidence.extend([
            "辛金生於卯月，卯藏乙木；課堂以偏財格作來源命例候選。".to_string(),
            "年、月兩見卯木，日支亥與卯可見木勢互動；課堂判作身弱財旺候選。".to_string(),
            "課堂取庚金比劫分財為用神方向、土印扶身為喜神方向；只作命例註記。".to_string(),
        ]);
        evidence.extend(
            interactions
                .iter()
                .filter(|interaction| {
                    matches!(interaction.target.as_str(), "亥卯" | "卯亥")
                        && interaction.r#type.contains("半合")
                })
                .map(|interaction| {
                    format!(
                        "互動證據：亥卯 {}（{}；{}；{}）。",
                        interaction.r#type,
                        interaction.outcome,
                        interaction.pillars,
                        interaction.detail
                    )
                }),
        );
    } else {
        evidence
            .push("精確命例 predicate：癸卯／乙卯／辛亥／庚寅，任一柱不同即不套用。".to_string());
    }

    QuantLessonInference {
        rule_id: "20260812-xin-wealth-case".to_string(),
        title: "2026/08/12 辛金偏財命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_and_whiteboard_confirmed".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: if !exact_case_matched {
            "not_comparable"
        } else {
            match (
                app_reference.ge_ju.contains("偏財"),
                app_reference.strength_label.contains('弱'),
            ) {
                (true, true) => "agree",
                (true, false) | (false, true) => "partial",
                (false, false) => "conflict",
            }
        }
        .to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: exact_case_matched.then(|| QuantLessonCandidate {
            pattern: "偏財格".to_string(),
            strength: "身弱財旺候選".to_string(),
            use_elements: vec!["金".to_string()],
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "課堂將本造列為偏財格、身弱財旺候選，以金比劫為用神方向、土印為喜神方向。App 普通格仍為「{}」，量化強弱、正式用神及特殊格均維持原值。",
                app_reference.ge_ju
            )
        } else {
            format!(
                "目前四柱「{}」不是本課例「癸卯 乙卯 辛亥 庚寅」，不套用偏財格、身弱財旺及金土方向候選。",
                chart
            )
        },
        evidence,
        rejected_alternatives: vec![
            "不把來源命例推廣成所有辛金卯月命造的正式規則。".to_string(),
            "課堂未定義可運算的旺衰門檻，因此不自動套用「旺神沖衰、衰神沖旺」於大運。".to_string(),
            "健康、婚姻、事故或其他敏感現實事件不納入自動推論。".to_string(),
        ],
    }
}

fn build_20260812_disease_medicine_case_inference(
    chart: &str,
    app_reference: &QuantLessonAppReference,
) -> QuantLessonInference {
    let exact_case_matched = chart == "壬戌 己酉 丁丑 甲辰";
    QuantLessonInference {
        rule_id: "20260812-disease-medicine-case".to_string(),
        title: "2026/08/12 財格病藥命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_exact".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: "not_comparable".to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: exact_case_matched.then(|| QuantLessonCandidate {
            pattern: "財格病藥命例".to_string(),
            strength: "課堂未另定強弱".to_string(),
            use_elements: vec!["木".to_string()],
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "課堂以己土食傷損官為病、甲木印制食傷護官為藥，木為病藥用神方向。App 普通格仍為「{}」，其量化結果不被覆蓋。",
                app_reference.ge_ju
            )
        } else {
            format!(
                "目前四柱「{}」不是本課例「壬戌 己酉 丁丑 甲辰」，不套用財格病藥與木用神方向候選。",
                chart
            )
        },
        evidence: if exact_case_matched {
            vec![
                "丁火生於酉月，課堂先以財格為命例背景。".to_string(),
                "己土食傷損官，課堂將此一作用標作「病」。".to_string(),
                "甲木印制食傷而護官，並可生助丁火，課堂將甲木標作「藥」。".to_string(),
            ]
        } else {
            vec![
                format!("目前四柱：{}。", chart),
                "精確命例 predicate：壬戌／己酉／丁丑／甲辰，任一柱不同即不套用。".to_string(),
            ]
        },
        rejected_alternatives: vec![
            "病與藥只表示命局中不利作用及其制衡，不對應醫療診斷。".to_string(),
            "不把此來源命例推廣成所有丁火酉月命造的正式病藥規則。".to_string(),
            "不以病藥候選覆蓋扶抑方向或 App 既有正式用神。".to_string(),
        ],
    }
}

fn build_20260812_follow_wealth_case_inference(
    chart: &str,
    app_reference: &QuantLessonAppReference,
    interactions: &[QuantModelInteraction],
) -> QuantLessonInference {
    let exact_case_matched = chart == "己卯 甲戌 乙未 庚辰";
    let mut evidence = vec![format!("目前四柱：{}。", chart)];
    if exact_case_matched {
        evidence.extend([
            "乙木生於戌月，課堂以戌月土旺及地支土勢作從財候選背景。".to_string(),
            "課堂將甲己合、卯戌合、乙庚合與戌未刑放在全局一併觀察，認為木的扶身作用不足。"
                .to_string(),
            "本課派結論為從財格候選：土財為用神、火食傷生財為喜神；只作來源命例註記。".to_string(),
        ]);
        evidence.extend(
            interactions
                .iter()
                .filter(|interaction| {
                    matches!(
                        interaction.target.as_str(),
                        "甲己" | "己甲" | "卯戌" | "戌卯" | "乙庚" | "庚乙" | "戌未" | "未戌"
                    )
                })
                .map(|interaction| {
                    let target = match interaction.target.as_str() {
                        "己甲" => "甲己",
                        "戌卯" => "卯戌",
                        "庚乙" => "乙庚",
                        "未戌" => "戌未",
                        other => other,
                    };
                    format!(
                        "互動證據：{} {}（{}；{}；{}）。",
                        target,
                        interaction.r#type,
                        interaction.outcome,
                        interaction.pillars,
                        interaction.detail
                    )
                }),
        );
    } else {
        evidence
            .push("精確命例 predicate：己卯／甲戌／乙未／庚辰，任一柱不同即不套用。".to_string());
    }

    QuantLessonInference {
        rule_id: "20260812-follow-wealth-case".to_string(),
        title: "2026/08/12 從財格命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_and_whiteboard_confirmed".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: if !exact_case_matched {
            "not_comparable"
        } else if app_reference.special_pattern.contains("從財") {
            "agree"
        } else {
            "conflict"
        }
        .to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: exact_case_matched.then(|| QuantLessonCandidate {
            pattern: "從財格".to_string(),
            strength: "本課派從弱候選".to_string(),
            use_elements: vec!["土".to_string()],
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "課堂將本造列為從財格候選，取土財為用神、火食傷生財為喜神。App 普通格仍為「{}」，量化強弱、正式用神及特殊格結果均保持原值並列比較。",
                app_reference.ge_ju
            )
        } else {
            format!(
                "目前四柱「{}」不是本課例「己卯 甲戌 乙未 庚辰」，不套用從財格及土火方向候選。",
                chart
            )
        },
        evidence,
        rejected_alternatives: vec![
            "本課例不足以建立通用從財 predicate、合化後根氣門檻或破格規則。".to_string(),
            "甲己合不涉及乙木日干本身，不直接改稱日主化土。".to_string(),
            "不覆蓋 App 的普通格、量化強弱、正式用神或既有特殊格結論。".to_string(),
            "疾病、人格、交友或其他敏感現實事件不納入自動推論。".to_string(),
        ],
    }
}

fn build_20260805_fu_yi_priority_inference(
    app_reference: &QuantLessonAppReference,
    yong_shen: &QuantYongShen,
) -> QuantLessonInference {
    QuantLessonInference {
        rule_id: "20260805-fu-yi-priority".to_string(),
        title: "扶抑用神優先（本課派候選對照）".to_string(),
        status: "已套用候選對照".to_string(),
        source_verification: "lesson_transcript_summary".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: "matched".to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "rule_validation".to_string(),
        agreement_status: "not_comparable".to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: Some(QuantLessonCandidate {
            pattern: "扶抑方向候選".to_string(),
            strength: format!("依 App 既有量化結果：{}", app_reference.strength_label),
            use_elements: yong_shen.favorable_elements.clone(),
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: format!(
            "依 2026/08/05 本課派扶抑規則，日主{}時以生扶為方向、日主強旺時以剋瀉耗為方向。此處只對照 App 既有量化用神「{}」，不另行選取或覆蓋正式用神。",
            app_reference.strength_label,
            join_chinese(&yong_shen.favorable_elements)
        ),
        evidence: vec![
            yong_shen.strength_basis.clone(),
            yong_shen.process.clone(),
            "課堂優先順序：扶抑優先於病藥、通關、調和；後三者不得與扶抑衝突。".to_string(),
        ],
        rejected_alternatives: vec![
            "不以病藥、通關或調和規則覆蓋 App 既有扶抑型量化用神。".to_string(),
            "不把課堂的扶抑說明當作另一套正式用神計分引擎。".to_string(),
            "健康、事故、感情、家族或其他敏感現實事件不納入自動推論。".to_string(),
        ],
    }
}

fn build_20260805_bing_fire_case_inference(
    chart: &str,
    app_reference: &QuantLessonAppReference,
) -> QuantLessonInference {
    let exact_case_matched = chart == "戊戌 己未 丙子 庚寅";
    QuantLessonInference {
        rule_id: "20260805-bing-fire-support-case".to_string(),
        title: "2026/08/05 丙火未月命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_exact".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: "not_comparable".to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: exact_case_matched.then(|| QuantLessonCandidate {
            pattern: "身弱候選".to_string(),
            strength: "本課派命例比較值".to_string(),
            use_elements: vec!["木".to_string(), "火".to_string()],
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "課堂將本造列為普通格身弱候選，木為剋土生火方向、火為幫身方向。App 普通格「{}」及量化用神維持原值，課堂值只作並列比較。",
                app_reference.ge_ju
            )
        } else {
            format!(
                "目前四柱「{}」不是本課例「戊戌 己未 丙子 庚寅」，不套用身弱、木火扶身的課堂候選值。",
                chart
            )
        },
        evidence: if exact_case_matched {
            vec![
                "課堂事實：丙火生未月，月令土氣增旺；己土本氣透干、戊戌再增土氣。".to_string(),
                "課堂事實：未、戌、寅皆有火根；時支寅木生丙火，因此不判為從格。".to_string(),
                "本課派結論：取木剋土生火為用、火幫身為喜；僅作來源命例候選。".to_string(),
            ]
        } else {
            vec![
                format!("目前四柱：{}。", chart),
                "精確命例 predicate：戊戌／己未／丙子／庚寅，任一柱不同即不套用。".to_string(),
            ]
        },
        rejected_alternatives: vec![
            "不以單一未月土旺就忽略未、戌、寅中的火根與寅木生火。".to_string(),
            "不覆蓋 App 正式用神；本課例的木火方向只作候選比較。".to_string(),
            "不從命例推出健康、感情、財富或保證式人生事件。".to_string(),
        ],
    }
}

fn build_20260722_day_master_state_inference(
    natal_cycles: &[SixtyCycle; 4],
    day_master: &str,
    app_reference: &QuantLessonAppReference,
) -> QuantLessonInference {
    let day_element = stem_element(day_master);
    let mother_element = mother_of(&day_element);
    let pressure_elements = vec![
        child_of(&day_element),
        wealth_element_of(&day_element),
        officer_element_of(&day_element),
    ];
    let branches = natal_cycles
        .iter()
        .map(|cycle| branch_name(cycle.get_earth_branch()))
        .collect::<Vec<_>>();
    let visible_stems = [0, 1, 3]
        .iter()
        .map(|index| stem_name(natal_cycles[*index].get_heaven_stem()))
        .collect::<Vec<_>>();
    let month_element = resolve_branch_element(&branches[1]);
    let obtains_time = month_element == day_element;
    let support_stem_count = visible_stems
        .iter()
        .filter(|stem| {
            let element = stem_element(stem);
            element == day_element || element == mother_element
        })
        .count();
    let support_branch_count = branches
        .iter()
        .filter(|branch| {
            let element = resolve_branch_element(branch);
            element == day_element || element == mother_element
        })
        .count();
    // 課堂只說「生扶比助多」；兩個可見來源是暫定工程門檻，因此整條規則保持 candidate。
    let obtains_support = support_stem_count >= 2 || support_branch_count >= 2;
    let root_count = branches
        .iter()
        .filter(|branch| {
            hidden_stems_of(branch)
                .iter()
                .any(|stem| stem_element(stem) == day_element)
        })
        .count();
    let obtains_root = root_count > 0;
    let pressure_stem_count = visible_stems
        .iter()
        .filter(|stem| pressure_elements.contains(&stem_element(stem)))
        .count();
    let pressure_branch_count = branches
        .iter()
        .filter(|branch| pressure_elements.contains(&resolve_branch_element(branch)))
        .count();
    let pressure_count = pressure_stem_count + pressure_branch_count;
    let factor_count = obtains_time as usize + obtains_support as usize + obtains_root as usize;
    let candidate_pattern = match (factor_count, pressure_count > 0) {
        (3, false) => "旺極候選",
        (3, true) => "旺候選",
        (2, _) => "強候選",
        (1, _) => "弱候選",
        (0, true) => "衰極候選",
        _ => "衰候選",
    };
    let use_elements = match candidate_pattern {
        "旺候選" | "強候選" => pressure_elements.clone(),
        "弱候選" | "衰候選" | "衰極候選" => {
            vec![mother_element.clone(), day_element.clone()]
        }
        _ => Vec::new(),
    };
    let factor_text = format!(
        "得時={}（月令{}）；得勢={}（生扶比助：天干{}、地支{}）；得地={}（同五行藏干根{}處）；克瀉耗可見{}處。",
        if obtains_time { "是" } else { "否" },
        month_element,
        if obtains_support { "是" } else { "否" },
        support_stem_count,
        support_branch_count,
        if obtains_root { "是" } else { "否" },
        root_count,
        pressure_count,
    );

    QuantLessonInference {
        rule_id: "20260722-day-master-state".to_string(),
        title: "日干得時／得勢／得地（本課派候選）".to_string(),
        status: "已套用候選規則".to_string(),
        source_verification: "lesson_transcript_summary".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: "matched".to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "rule_validation".to_string(),
        agreement_status: "not_comparable".to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: Some(QuantLessonCandidate {
            pattern: candidate_pattern.to_string(),
            strength: "本課派日干狀態候選".to_string(),
            use_elements,
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: format!(
            "本課派候選為「{}」。{} App 的普通格、量化強弱、用神與特殊格結果只作並列參照，不直接覆蓋。",
            candidate_pattern, factor_text
        ),
        evidence: vec![
            factor_text,
            "規則來源：2026/07/22 課程以得時、得勢、得地及克瀉耗判日干旺、強、弱、衰與兩端狀態。".to_string(),
            "得勢的兩來源門檻是 App 工程候選，課程原文未給可驗證數值，因此規則維持 candidate。".to_string(),
        ],
        rejected_alternatives: vec![
            "不直接覆蓋 App 的普通格、量化強弱、用神或既有特殊格結論。".to_string(),
            "旺極或衰極不自動改判專旺／從格；仍須通過既有特殊格的獨旺、破格與回歸檢查。".to_string(),
            "健康、婚姻、死亡或保證式現實事件不納入自動推論。".to_string(),
        ],
    }
}

fn build_20260729_geng_shen_case_inference(
    chart: &str,
    app_reference: &QuantLessonAppReference,
    interactions: &[QuantModelInteraction],
) -> QuantLessonInference {
    let exact_case_matched = chart == "丙子 丙申 庚寅 丁亥";
    let interaction_evidence = interactions
        .iter()
        .filter(|interaction| {
            interaction.r#type.contains("半合") || interaction.r#type.contains("沖")
        })
        .map(|interaction| {
            let target = match interaction.target.as_str() {
                "子申" | "申子" => "申子",
                "申寅" | "寅申" => "寅申",
                other => other,
            };
            format!(
                "互動證據：{} {}（{}；{}；{}）。",
                target,
                interaction.r#type,
                interaction.outcome,
                interaction.pillars,
                interaction.detail
            )
        })
        .collect::<Vec<_>>();

    let mut evidence = vec![
        format!("目前四柱：{}。", chart),
        "課堂來源：庚金生申月先記得時，再檢查根氣與生剋瀉合化的變化。".to_string(),
    ];
    evidence.extend(interaction_evidence);
    if evidence.len() == 2 {
        evidence.push("本命未產生可供本課例使用的半合或沖互動列。".to_string());
    }

    QuantLessonInference {
        rule_id: "20260729-geng-shen-weak-case".to_string(),
        title: "2026/07/29 庚金申月命例比對".to_string(),
        status: if exact_case_matched {
            "已比對命例"
        } else {
            "未符合本課命例"
        }
        .to_string(),
        source_verification: "transcript_exact".to_string(),
        algorithm_status: "candidate".to_string(),
        match_status: if exact_case_matched {
            "matched"
        } else {
            "not_matched"
        }
        .to_string(),
        output_mode: "annotation_only".to_string(),
        fixture_scope: "source_case_only".to_string(),
        agreement_status: "not_comparable".to_string(),
        fallback_policy: "preserve_normal_outputs".to_string(),
        app_reference: app_reference.clone(),
        lesson_candidate: exact_case_matched.then(|| QuantLessonCandidate {
            pattern: "日干弱候選".to_string(),
            strength: "本課派比較值".to_string(),
            use_elements: Vec::new(),
        }),
        sensitive_claims_policy: "excluded".to_string(),
        conclusion: if exact_case_matched {
            format!(
                "課堂將本造列為日干弱候選；App 普通格仍為「{}」，量化強弱、用神與特殊格保持原值，兩者只作並列比較。",
                app_reference.ge_ju
            )
        } else {
            format!(
                "目前四柱「{}」不是本課例「丙子 丙申 庚寅 丁亥」，不套用日干弱候選。App 普通格「{}」與其他量化結果保持原值。",
                chart, app_reference.ge_ju
            )
        },
        evidence,
        rejected_alternatives: vec![
            "不因庚金生申月得時就跳過根氣、半合與沖的互動檢查。".to_string(),
            "不把來源命例的日干弱候選推廣成所有庚金申月命例的正式身弱規則。".to_string(),
            "不覆蓋 App 的普通格、量化強弱、用神或既有特殊格結論。".to_string(),
            "健康、事故、家庭或其他敏感現實事件不納入自動推論。".to_string(),
        ],
    }
}

fn build_natal_inputs(natal_cycles: &[SixtyCycle; 4]) -> CalculationInputs {
    let stems = vec![
        StemInput {
            pillar: "年干".to_string(),
            stem: stem_name(natal_cycles[0].get_heaven_stem()),
            position_index: 0,
        },
        StemInput {
            pillar: "月干".to_string(),
            stem: stem_name(natal_cycles[1].get_heaven_stem()),
            position_index: 1,
        },
        StemInput {
            pillar: "時干".to_string(),
            stem: stem_name(natal_cycles[3].get_heaven_stem()),
            position_index: 3,
        },
    ];
    let branches = ["年支", "月支", "日支", "時支"]
        .iter()
        .enumerate()
        .map(|(index, pillar)| {
            let branch = branch_name(natal_cycles[index].get_earth_branch());
            BranchInput {
                pillar: (*pillar).to_string(),
                branch: branch.clone(),
                hidden_stems: hidden_stems_of(&branch),
                position_index: index,
            }
        })
        .collect();
    CalculationInputs { stems, branches }
}

fn score_resolved_chart(day_master: &str, inputs: &CalculationInputs) -> ResolvedScoreBundle {
    let resolved_state = resolve_chart_state(day_master, inputs);
    let stem_scores = score_stems(day_master, &resolved_state);
    let branch_scores = score_branches(day_master, &resolved_state);
    ResolvedScoreBundle {
        resolved_state,
        stem_scores,
        branch_scores,
    }
}

fn resolve_chart_state(day_master: &str, inputs: &CalculationInputs) -> ResolvedChartState {
    let three_meeting_candidates =
        resolve_three_meeting_candidates(&inputs.branches, &inputs.stems, day_master);
    let three_combination_candidates =
        resolve_three_combination_candidates(&inputs.branches, &inputs.stems, day_master);
    let half_combination_candidates =
        resolve_half_combination_candidates(&inputs.branches, &inputs.stems, day_master);

    let mut branch_transforms =
        resolve_four_storage_transforms(&inputs.branches, &inputs.stems, day_master);
    merge_missing_transforms(
        &mut branch_transforms,
        resolve_three_meeting_transforms(&three_meeting_candidates, &inputs.branches),
    );
    merge_missing_transforms(
        &mut branch_transforms,
        resolve_three_combination_transforms(
            &three_combination_candidates,
            &three_meeting_candidates,
            &inputs.branches,
        ),
    );
    merge_missing_transforms(
        &mut branch_transforms,
        resolve_half_combination_transforms(
            &half_combination_candidates,
            &three_combination_candidates,
            &three_meeting_candidates,
            &inputs.branches,
            &inputs.stems,
            day_master,
        ),
    );
    let current_transforms = branch_transforms.clone();
    merge_transforms(
        &mut branch_transforms,
        resolve_six_combination_transforms(
            &inputs.branches,
            &inputs.stems,
            day_master,
            &half_combination_candidates,
            &three_combination_candidates,
            &three_meeting_candidates,
            &current_transforms,
        ),
        &inputs.branches,
    );

    let mut six_combination_effects = resolve_six_combination_effects(
        &inputs.branches,
        &half_combination_candidates,
        &three_combination_candidates,
        &three_meeting_candidates,
        &branch_transforms,
    );
    let six_clash_resolution = resolve_six_combination_clash_cushions(
        &inputs.branches,
        &mut six_combination_effects,
        &branch_transforms,
    );
    let branch_prune_effects = resolve_branch_prune_effects(&inputs.branches, &branch_transforms);
    let non_transform_boosts = resolve_non_transform_boosts(
        &inputs.branches,
        &three_combination_candidates,
        &three_meeting_candidates,
        &branch_transforms,
    );
    let branch_qi_modifiers = resolve_branch_qi_modifiers(
        &inputs.branches,
        &six_combination_effects,
        &three_meeting_candidates,
        &branch_transforms,
        &six_clash_resolution,
        day_master,
    );
    let mut stem_combination_effects = resolve_stem_combination_effects(
        &inputs.stems,
        &inputs.branches[1],
        branch_transforms.get(&1),
    );
    let stem_clash_effects =
        resolve_stem_clash_effects(&inputs.stems, &mut stem_combination_effects);

    ResolvedChartState {
        stems: inputs.stems.clone(),
        branches: inputs.branches.clone(),
        three_meeting_candidates,
        three_combination_candidates,
        half_combination_candidates,
        branch_transforms,
        six_combination_effects,
        branch_prune_effects,
        branch_qi_modifiers,
        stem_combination_effects,
        stem_clash_effects,
        six_clash_resolution,
        non_transform_boosts,
    }
}

fn score_stems(day_master: &str, state: &ResolvedChartState) -> Vec<QuantModelPillarScore> {
    state
        .stems
        .iter()
        .map(|input| {
            let stem_combination_effect = state.stem_combination_effects.get(&input.position_index);
            let stem_clash_effect = state.stem_clash_effects.get(&input.position_index);
            let effective_stem = if let Some(effect) = stem_combination_effect {
                if effect.transforms {
                    stem_for_element(&effect.target_element, is_yang(&input.stem))
                } else {
                    input.stem.clone()
                }
            } else {
                input.stem.clone()
            };
            let ten_god = resolve_ten_god(day_master, &effective_stem);
            let raw_score = resolve_ten_god_score(&ten_god);
            let weight = position_weight(&input.pillar);
            let mut position_adjusted = round1(raw_score / 100.0 * weight);
            let mut combine_note = None;
            let mut details = vec![format!(
                "{} 對 {} 為 {}（{:.0}），位置權重 {:.0}，位調後 {:.1}",
                effective_stem, day_master, ten_god, raw_score, weight, position_adjusted
            )];
            if let Some(effect) = stem_combination_effect {
                position_adjusted = round1(position_adjusted * effect.remainder);
                combine_note = Some(effect.combine_note.clone());
                details.push(format!(
                    "{}，位調分數×{}",
                    effect.note,
                    round3_string(effect.remainder)
                ));
            }
            if let Some(effect) = stem_clash_effect {
                position_adjusted = round1(position_adjusted * effect.remainder);
                details.push(format!(
                    "{}，位調分數×{}",
                    effect.note,
                    round3_string(effect.remainder)
                ));
            }

            let mut rows = Vec::new();
            let mut total_root_bonus = 0.0;
            for branch in &state.branches {
                let root = resolve_stem_root(
                    &effective_stem,
                    branch,
                    state.branch_transforms.get(&branch.position_index),
                    state.branch_prune_effects.get(&branch.position_index),
                    state.branch_qi_modifiers.get(&branch.position_index),
                );
                if root.level == RootLevel::None {
                    continue;
                }
                let mut multiplier = root_multiplier(
                    root.level,
                    stem_branch_distance(input, branch),
                ) * root.qi_multiplier;
                let mut extra_reason = String::new();
                if let Some(effect) = state.six_combination_effects.get(&branch.position_index) {
                    multiplier *= effect.remainder;
                    extra_reason = format!("，{}六合而不化餘力", effect.pair);
                }
                let bonus = round1(position_adjusted * multiplier);
                total_root_bonus += bonus;
                rows.push(QuantModelSubScore {
                    item: "得根".to_string(),
                    hidden_stem: Some(root.support_stem.clone()),
                    ten_god: Some(ten_god.clone()),
                    ratio: None,
                    raw_score: Some(raw_score),
                    position_adjusted_score: Some(round1(position_adjusted)),
                    interaction: Some(format!(
                        "{}@{}{}",
                        root.level.label(),
                        branch.pillar,
                        branch.branch
                    )),
                    adjustment_score: Some(round1(bonus)),
                    final_contribution: Some(round1(position_adjusted + bonus)),
                    note: Some(format!(
                        "根氣增值 ×{}（{}{}）",
                        round3_string(multiplier),
                        root.reason,
                        extra_reason
                    )),
                });
                details.push(format!(
                    "得{}於{}{}（{}{}，×{}，{:+.1}）",
                    root.level.label(),
                    branch.pillar,
                    branch.branch,
                    root.reason,
                    extra_reason,
                    round3_string(multiplier),
                    round1(bonus)
                ));
            }

            QuantModelPillarScore {
                pillar: input.pillar.clone(),
                target: input.stem.clone(),
                ten_god,
                base_score: raw_score,
                position_weight: weight,
                final_score: round1(position_adjusted + total_root_bonus),
                details: details.join("；"),
                category: Some("STEM".to_string()),
                combine_note,
                raw_score: Some(raw_score),
                position_adjusted_score: Some(round1(position_adjusted)),
                clash_adjustment: Some(0.0),
                bonus_score: Some(round1(total_root_bonus)),
                rows: Some(rows),
            }
        })
        .collect()
}

fn score_branches(day_master: &str, state: &ResolvedChartState) -> Vec<QuantModelPillarScore> {
    state
        .branches
        .iter()
        .map(|input| score_branch(day_master, input, state))
        .collect()
}

fn score_branch(
    day_master: &str,
    input: &BranchInput,
    state: &ResolvedChartState,
) -> QuantModelPillarScore {
    let weight = position_weight(&input.pillar);
    let branch_transform = state.branch_transforms.get(&input.position_index);
    let branch_prune_effect = state.branch_prune_effects.get(&input.position_index);
    let branch_qi_modifier = state.branch_qi_modifiers.get(&input.position_index);
    let hidden_stems = if let Some(effect) = branch_prune_effect {
        vec![effect.remaining_stem.clone()]
    } else if input.hidden_stems.is_empty() {
        hidden_stems_of(&input.branch)
    } else {
        input.hidden_stems.clone()
    };
    let ratios = if let Some(effect) = branch_prune_effect {
        vec![effect.remaining_ratio]
    } else {
        hidden_ratios(hidden_stems.len())
    };

    let branch_interaction = resolve_branch_interaction(input, state);
    let mut details = Vec::new();
    if let Some(transform) = branch_transform {
        details.push(format!(
            "{}：該支改以化後五行計分，並優先於其他沖合與原藏干",
            transform.note
        ));
    }
    if let Some(effect) = branch_prune_effect {
        details.push(effect.note.clone());
    }
    if let Some(modifier) = branch_qi_modifier {
        details.push(modifier.note.clone());
    }
    if (branch_interaction.multiplier - 1.0).abs() > 0.0001 {
        details.push(format!(
            "先做沖合耗損：{}，支內各分項×{}",
            branch_interaction.note,
            round3_string(branch_interaction.multiplier)
        ));
    }

    let mut slices = if let Some(transform) = branch_transform {
        build_transformed_branch_qi_slices(input, transform, day_master, weight)
    } else {
        build_natural_branch_qi_slices(
            input,
            day_master,
            weight,
            &hidden_stems,
            &ratios,
            &branch_interaction,
            branch_qi_modifier,
        )
    };
    let boost_slices = build_boost_branch_qi_slices(
        input,
        state.non_transform_boosts.get(&input.position_index),
        day_master,
        weight,
    );
    let boost_raw_total: f64 = boost_slices
        .iter()
        .map(|slice| slice.adjusted_raw_contribution)
        .sum();
    let boost_position_total: f64 = boost_slices.iter().map(|slice| slice.position_adjusted).sum();
    slices.extend(boost_slices);

    let mut rows = Vec::new();
    let mut adjusted_raw_total = 0.0;
    let mut position_adjusted_total = 0.0;
    let mut transparency_bonus_total = 0.0;
    for slice in slices {
        adjusted_raw_total += slice.adjusted_raw_contribution;
        position_adjusted_total += slice.position_adjusted;
        let transparencies = if slice.transformed {
            resolve_transformed_branch_transparencies(
                &slice.transparency_key,
                slice.transform_slice_kind,
                input,
                &state.stems,
                &state.stem_combination_effects,
                &state.stem_clash_effects,
                slice.transparency_base,
            )
        } else {
            resolve_branch_transparencies(
                &slice.transparency_key,
                slice.hidden_stem_index,
                input,
                &state.stems,
                &state.stem_combination_effects,
                &state.stem_clash_effects,
                &state.branch_prune_effects,
                slice.position_adjusted,
            )
        };
        let transparency_bonus: f64 = transparencies.iter().map(|item| item.bonus).sum();
        transparency_bonus_total += transparency_bonus;
        let interaction = if transparencies.is_empty() {
            slice.base_interaction.clone()
        } else {
            transparencies
                .iter()
                .map(TransparencyApplication::label)
                .collect::<Vec<_>>()
                .join("；")
        };
        let note = if transparencies.is_empty() {
            slice.base_note.clone()
        } else {
            format!(
                "{}；{}",
                slice.base_note,
                transparencies
                    .iter()
                    .map(|item| format!(
                        "{}於{}（×{}，{:+.1}）",
                        item.kind,
                        item.visible_stem,
                        round3_string(item.multiplier),
                        round1(item.bonus)
                    ))
                    .collect::<Vec<_>>()
                    .join("；")
            )
        };
        details.push(note.clone());
        rows.push(QuantModelSubScore {
            item: input.pillar.clone(),
            hidden_stem: Some(slice.display_stem.clone()),
            ten_god: Some(slice.ten_god.clone()),
            ratio: Some(round3(slice.ratio)),
            raw_score: Some(round1(slice.row_raw_score)),
            position_adjusted_score: Some(round1(slice.position_adjusted)),
            interaction: Some(interaction),
            adjustment_score: Some(round1(transparency_bonus)),
            final_contribution: Some(round1(slice.position_adjusted + transparency_bonus)),
            note: Some(note),
        });
    }

    let lu_element = stem_element(day_master);
    let lu_stem = stem_for_element(&lu_element, is_yang(day_master));
    // 祿支內任一藏干之氣受沖化零，就不再整支 +100，只按剩餘藏干計分
    // （八字真理上冊 p30 戊申日巳月、p71 己亥日巳月：巳只剩戊 +25）。
    let lu_branch_qi_zeroed = hidden_stems_of(&input.branch).iter().any(|stem| {
        branch_score_qi_multiplier(input, stem, &branch_interaction, branch_qi_modifier) <= 0.0
    });
    let lu_qi_multiplier = if lu_branch_qi_zeroed {
        0.0
    } else if let Some(main_qi) =
        branch_qi_modifier.and_then(|modifier| modifier.stem_multipliers.get(LU_MAIN_QI_KEY))
    {
        *main_qi
    } else {
        branch_score_qi_multiplier(input, &lu_stem, &branch_interaction, branch_qi_modifier)
    };
    if branch_transform.is_none()
        && is_element_lu_branch(day_master, &input.branch)
        && lu_qi_multiplier <= 0.0
    {
        details.push("祿支藏干受沖化零：不再以 +100 計，改按剩餘藏干計分".to_string());
    }
    if branch_transform.is_none()
        && is_element_lu_branch(day_master, &input.branch)
        && lu_qi_multiplier > 0.0
    {
        adjusted_raw_total = 100.0 + boost_raw_total;
        let qi_multiplier = lu_qi_multiplier;
        position_adjusted_total = round1(
            weight * branch_interaction.multiplier * qi_multiplier + boost_position_total,
        );
        details.push("地支到祿：以 +100 直接取代各藏干合值，再另加透干增值".to_string());
        rows = vec![QuantModelSubScore {
            item: input.pillar.clone(),
            hidden_stem: Some("祿".to_string()),
            ten_god: Some(resolve_element_ten_god(day_master, &lu_element)),
            ratio: Some(1.0),
            raw_score: Some(100.0),
            position_adjusted_score: Some(position_adjusted_total),
            interaction: Some(if branch_interaction.note.is_empty() {
                "地支到祿".to_string()
            } else {
                branch_interaction.note.clone()
            }),
            adjustment_score: Some(round1(transparency_bonus_total)),
            final_contribution: Some(round1(position_adjusted_total + transparency_bonus_total)),
            note: Some("地支到祿：以祿支整體氣勢計分。".to_string()),
        }];
    }

    QuantModelPillarScore {
        pillar: input.pillar.clone(),
        target: input.branch.clone(),
        ten_god: if let Some(transform) = branch_transform {
            resolve_element_ten_god(day_master, &transform.element)
        } else {
            hidden_stems.join("/")
        },
        base_score: round1(adjusted_raw_total),
        position_weight: weight,
        final_score: round1(position_adjusted_total + transparency_bonus_total),
        details: details.join("；"),
        category: Some("BRANCH".to_string()),
        combine_note: if branch_interaction.combine_note.is_empty() {
            branch_transform.map(|item| item.combine_note.clone())
        } else {
            Some(branch_interaction.combine_note)
        },
        raw_score: Some(round1(adjusted_raw_total)),
        position_adjusted_score: Some(round1(position_adjusted_total)),
        clash_adjustment: Some(if (branch_interaction.multiplier - 1.0).abs() < 0.0001 {
            0.0
        } else {
            round1(branch_interaction.multiplier)
        }),
        bonus_score: Some(round1(transparency_bonus_total)),
        rows: Some(rows),
    }
}

fn build_natural_branch_qi_slices(
    input: &BranchInput,
    day_master: &str,
    weight: f64,
    hidden_stems: &[String],
    ratios: &[f64],
    branch_interaction: &BranchInteraction,
    branch_qi_modifier: Option<&BranchQiModifier>,
) -> Vec<BranchQiSlice> {
    hidden_stems
        .iter()
        .enumerate()
        .map(|(index, hidden_stem)| {
            let ten_god = resolve_ten_god(day_master, hidden_stem);
            let raw_score =
                resolve_branch_hidden_stem_score(day_master, &input.branch, hidden_stem, &ten_god);
            let qi_multiplier = branch_score_qi_multiplier(
                input,
                hidden_stem,
                branch_interaction,
                branch_qi_modifier,
            );
            let adjusted_raw =
                raw_score * ratios[index] * branch_interaction.multiplier * qi_multiplier;
            let position_adjusted = round1(adjusted_raw / 100.0 * weight);
            BranchQiSlice {
                display_stem: hidden_stem.clone(),
                ten_god,
                ratio: ratios[index],
                row_raw_score: raw_score * ratios[index],
                adjusted_raw_contribution: adjusted_raw,
                position_adjusted,
                base_interaction: if branch_interaction.note.is_empty() {
                    "原局".to_string()
                } else {
                    branch_interaction.note.clone()
                },
                base_note: format!(
                    "{}={} {:.0}×{:.2}{}{} → 位調後 {:.1}",
                    hidden_stem,
                    resolve_ten_god(day_master, hidden_stem),
                    raw_score,
                    ratios[index],
                    if (branch_interaction.multiplier - 1.0).abs() < 0.0001 {
                        "".to_string()
                    } else {
                        format!("×{}", round3_string(branch_interaction.multiplier))
                    },
                    if (qi_multiplier - 1.0).abs() < 0.0001 {
                        "".to_string()
                    } else {
                        format!("×{}", round3_string(qi_multiplier))
                    },
                    position_adjusted
                ),
                transformed: false,
                hidden_stem_index: index as isize,
                transparency_key: hidden_stem.clone(),
                transparency_base: position_adjusted,
                transform_slice_kind: BranchTransformSliceKind::Residual,
            }
        })
        .collect()
}

fn build_transformed_branch_qi_slices(
    input: &BranchInput,
    branch_transform: &BranchTransform,
    day_master: &str,
    weight: f64,
) -> Vec<BranchQiSlice> {
    branch_transform
        .slices
        .iter()
        .map(|slice| {
            if slice.transformed {
                let ten_god = resolve_element_ten_god(day_master, &slice.display_stem);
                let base_element_score = resolve_element_score(day_master, &slice.display_stem);
                let transformed_raw_score = base_element_score * slice.factor;
                let position_adjusted = round1(transformed_raw_score / 100.0 * weight);
                let transparency_base =
                    base_element_score / 100.0 * weight * slice.transparency_base_factor;
                BranchQiSlice {
                    display_stem: slice.display_stem.clone(),
                    ten_god,
                    ratio: 1.0,
                    row_raw_score: transformed_raw_score,
                    adjusted_raw_contribution: transformed_raw_score,
                    position_adjusted,
                    base_interaction: slice.interaction.clone(),
                    base_note: slice.note.clone(),
                    transformed: true,
                    hidden_stem_index: -1,
                    transparency_key: slice.display_stem.clone(),
                    transparency_base,
                    transform_slice_kind: slice.kind,
                }
            } else {
                let ten_god = resolve_ten_god(day_master, &slice.display_stem);
                let original_raw = resolve_branch_hidden_stem_score(
                    day_master,
                    &input.branch,
                    &slice.display_stem,
                    &ten_god,
                );
                let raw_residual = original_raw * slice.ratio * slice.factor;
                let position_adjusted = round1(raw_residual / 100.0 * weight);
                BranchQiSlice {
                    display_stem: slice.display_stem.clone(),
                    ten_god,
                    ratio: slice.ratio * slice.factor,
                    row_raw_score: raw_residual,
                    adjusted_raw_contribution: raw_residual,
                    position_adjusted,
                    base_interaction: slice.interaction.clone(),
                    base_note: format!(
                        "{}餘氣={} {:.0}×{:.2}×{} → 位調後 {:.1}",
                        input.branch,
                        resolve_ten_god(day_master, &slice.display_stem),
                        original_raw,
                        slice.ratio,
                        round3_string(slice.factor),
                        position_adjusted
                    ),
                    transformed: false,
                    hidden_stem_index: slice.hidden_stem_index,
                    transparency_key: slice.display_stem.clone(),
                    transparency_base: position_adjusted,
                    transform_slice_kind: slice.kind,
                }
            }
        })
        .collect()
}

/// 大運干支依次入局（下冊 p202）：運干掌前五年、運支掌後五年，各自與原局同場裁決，
/// 運干不得根於運支，運支亦不透於運干。
fn build_luck_scores(
    day_master: &str,
    natal_total: f64,
    natal_inputs: &CalculationInputs,
    da_yun: &[DaYunDto],
) -> Vec<QuantLuckScore> {
    let natal_cells_base = natal_cells_total(&score_resolved_chart(day_master, natal_inputs));
    da_yun
        .iter()
        .filter_map(|item| {
            let (luck_stem, luck_branch) = split_gan_zhi(&item.gan_zhi)?;
            let stem_bundle = score_resolved_chart(
                day_master,
                &build_extended_inputs(natal_inputs, Some(&luck_stem), None, None, None),
            );
            let branch_bundle = score_resolved_chart(
                day_master,
                &build_extended_inputs(natal_inputs, None, Some(&luck_branch), None, None),
            );
            let luck_stem_pillar = resolve_pillar_score(&stem_bundle.stem_scores, "運干");
            let luck_branch_pillar = resolve_pillar_score(&branch_bundle.branch_scores, "運支");
            let stem_score = luck_stem_pillar.map(|item| item.final_score).unwrap_or(0.0);
            let branch_score = luck_branch_pillar
                .map(|item| item.final_score)
                .unwrap_or(0.0);
            let total_score = round1((stem_score + branch_score) / 2.0);
            // 大運入局對原局的影響（八字真理上冊 p70）：R 值用「運本身得分 + 原局因此而變的分數」。
            let stem_natal_shift = natal_cells_total(&stem_bundle) - natal_cells_base;
            let branch_natal_shift = natal_cells_total(&branch_bundle) - natal_cells_base;
            let stem_impact = round1(stem_score + stem_natal_shift);
            let branch_impact = round1(branch_score + branch_natal_shift);
            let total_impact = round1((stem_impact + branch_impact) / 2.0);
            let impact_ratio = luck_impact_ratio(total_impact, natal_total);
            let effective_natal_score = luck_effective_natal_score(natal_total, impact_ratio);
            let stem_ratio = luck_impact_ratio(stem_impact, natal_total);
            let branch_ratio = luck_impact_ratio(branch_impact, natal_total);
            let mut overview_lines = vec![
                format!(
                    "前五年看運干：{:.1}（R值 {:.2}，作用後命局 {:.1}）",
                    stem_score,
                    stem_ratio,
                    luck_effective_natal_score(natal_total, stem_ratio)
                ),
                format!(
                    "後五年看運支：{:.1}（R值 {:.2}，作用後命局 {:.1}）",
                    branch_score,
                    branch_ratio,
                    luck_effective_natal_score(natal_total, branch_ratio)
                ),
            ];
            for (label, shift, impact) in [
                ("運干", stem_natal_shift, stem_impact),
                ("運支", branch_natal_shift, branch_impact),
            ] {
                if shift.abs() >= 0.05 {
                    overview_lines.push(format!(
                        "{}入局令原局變化 {:+.1}，整體衝擊 {:.1}（R 值以此計）",
                        label, shift, impact
                    ));
                }
            }
            overview_lines.push(format!("整柱參考（前後五年平均）：{:.1}", total_score));
            overview_lines.push(format!("R值：{:.2}", impact_ratio));
            overview_lines.push(format!("作用後命局：{:.1}", effective_natal_score));
            overview_lines.extend(luck_wandering_lines(&stem_bundle.resolved_state, "運干"));
            overview_lines.extend(luck_wandering_lines(&branch_bundle.resolved_state, "運支"));
            let annual_scores = if item.liu_nian.is_empty() {
                None
            } else {
                Some(build_annual_scores(
                    day_master,
                    natal_total,
                    natal_cells_base,
                    natal_inputs,
                    item,
                ))
            };

            Some(QuantLuckScore {
                index: item.index,
                gan_zhi: item.gan_zhi.clone(),
                start_age: item.start_age,
                end_age: item.end_age,
                stem_ten_god: item.stem_ten_god.clone(),
                branch_hidden_stems: item.branch_hidden_stems.clone(),
                branch_ten_gods: item.branch_ten_gods.clone(),
                first_half_label: format!("前五年看運干{}", luck_stem),
                first_half_score: round1(stem_score),
                second_half_label: format!("後五年看運支{}", luck_branch),
                second_half_score: round1(branch_score),
                stem_score: round1(stem_score),
                branch_score: round1(branch_score),
                total_score,
                impact_ratio,
                effective_natal_score,
                tendency: tendency_label(total_score, natal_total),
                overview_lines: Some(overview_lines),
                interaction_lines: Some(build_pillar_detail_lines(
                    luck_stem_pillar,
                    luck_branch_pillar,
                )),
                scoring_lines: Some(vec![
                    format!("運干列分（位置分值 60）：{:.1}", stem_score),
                    format!("運支列分（位置分值 60）：{:.1}", branch_score),
                    "R = L / √(M² + L²)，L = 運本身得分 + 原局因運而變的分數；身強 M′ = M(1+R)，身弱 M′ = M(1−R)".to_string(),
                ]),
                details: format!(
                    "運干 {} 與運支 {} 依次入局，各自與原局同場裁決；入局干支得根透干按同柱、沖合按緊貼計。",
                    luck_stem, luck_branch
                ),
                annual_scores,
            })
        })
        .collect()
}

/// 原局七格（不含運、歲）的分數合計。
fn natal_cells_total(bundle: &ResolvedScoreBundle) -> f64 {
    bundle
        .stem_scores
        .iter()
        .chain(bundle.branch_scores.iter())
        .filter(|item| !is_floating_pillar(&item.pillar))
        .map(|item| item.final_score)
        .sum()
}

/// 游走的大運（八字真理上冊 p58-60）：運干在天干間、運支在地支間由年柱游到時柱。
/// 計分仍按緊貼（量化五行力下冊 p202），這裡只補充說明游到各位置時的沖力：
/// 緊貼或同位化零、隔一位 ×0.75、隔二位 ×8/9，各位置約佔四分之一時間；
/// 與天干合而不化則被合住在該位置，不再游走（半合不化仍會游走）。
fn luck_wandering_lines(state: &ResolvedChartState, luck_pillar: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if luck_pillar == "運干" {
        let Some(luck) = state.stems.iter().find(|item| item.pillar == luck_pillar) else {
            return lines;
        };
        let natal: Vec<&StemInput> = state
            .stems
            .iter()
            .filter(|item| !is_floating_pillar(&item.pillar))
            .collect();
        if let Some(effect) = state.stem_combination_effects.get(&luck.position_index) {
            if !effect.transforms {
                let partners: Vec<String> = natal
                    .iter()
                    .filter(|item| resolve_stem_combination(&luck.stem, &item.stem).is_some())
                    .map(|item| format!("{}{}", item.pillar, item.stem))
                    .collect();
                if !partners.is_empty() {
                    lines.push(format!(
                        "運干{}與{}合而不化，被合住在該位置，不再游走（八字真理上冊 p60）",
                        luck.stem,
                        partners.join("、")
                    ));
                    return lines;
                }
            }
        }
        for item in &natal {
            if is_stem_clash(&luck.stem, &item.stem) {
                lines.push(wandering_clash_line(
                    "運干",
                    &luck.stem,
                    &item.pillar,
                    &item.stem,
                    item.position_index,
                ));
            }
        }
    } else {
        let Some(luck) = state.branches.iter().find(|item| item.pillar == luck_pillar) else {
            return lines;
        };
        if state.branch_transforms.contains_key(&luck.position_index) {
            return lines;
        }
        let natal: Vec<&BranchInput> = state
            .branches
            .iter()
            .filter(|item| !is_floating_pillar(&item.pillar))
            .collect();
        for item in &natal {
            if is_direct_clash(&luck.branch, &item.branch) {
                lines.push(wandering_clash_line(
                    "運支",
                    &luck.branch,
                    &item.pillar,
                    &item.branch,
                    item.position_index,
                ));
            }
        }
    }
    lines
}

fn wandering_clash_line(label: &str, luck: &str, pillar: &str, target: &str, slot: usize) -> String {
    let mut adjacent = 0;
    let mut one_apart = 0;
    let mut two_apart = 0;
    for position in 0..4usize {
        match position.abs_diff(slot) {
            0 | 1 => adjacent += 1,
            2 => one_apart += 1,
            _ => two_apart += 1,
        }
    }
    let average = (one_apart as f64 * 0.75 + two_apart as f64 * 8.0 / 9.0) / 4.0;
    let mut parts = vec![format!("約 {}/4 時間緊貼化零", adjacent)];
    if one_apart > 0 {
        parts.push(format!("約 {}/4 時間隔一位剩 0.75", one_apart));
    }
    if two_apart > 0 {
        parts.push(format!("約 {}/4 時間隔二位剩 8/9", two_apart));
    }
    format!(
        "游走的大運：{}{}與{}{}相沖，{}；平均約剩 {:.2}（計分仍按緊貼）",
        label,
        luck,
        pillar,
        target,
        parts.join("、"),
        average
    )
}

/// 大運對命局的相對影響（下冊 p199）：R = L / √(M² + L²)。
fn luck_impact_ratio(luck_score: f64, natal_total: f64) -> f64 {
    let base = (natal_total * natal_total + luck_score * luck_score).sqrt();
    if base < 0.1 {
        0.0
    } else {
        round2(luck_score / base)
    }
}

/// 身強 M′ = M(1+R)、身弱 M′ = M(1−R)；大運不能使命局由正變負或由負變正（下冊 p199-200）。
fn luck_effective_natal_score(natal_total: f64, impact_ratio: f64) -> f64 {
    if natal_total >= 0.0 {
        round1(natal_total * (1.0 + impact_ratio))
    } else {
        round1(natal_total * (1.0 - impact_ratio))
    }
}

fn build_annual_scores(
    day_master: &str,
    natal_total: f64,
    natal_cells_base: f64,
    natal_inputs: &CalculationInputs,
    da_yun: &DaYunDto,
) -> Vec<QuantAnnualLuckScore> {
    let Some((luck_stem, luck_branch)) = split_gan_zhi(&da_yun.gan_zhi) else {
        return Vec::new();
    };
    da_yun
        .liu_nian
        .iter()
        .filter_map(|item| {
            build_annual_score(
                day_master,
                natal_total,
                natal_cells_base,
                natal_inputs,
                &luck_stem,
                &luck_branch,
                item,
            )
        })
        .collect()
}

/// 某年運程 = 當值大運（干或支，60）入局之剋應 + 歲運干支（各 20）同時入局之剋應（下冊 p197-198、p202）。
fn build_annual_score(
    day_master: &str,
    natal_total: f64,
    natal_cells_base: f64,
    natal_inputs: &CalculationInputs,
    luck_stem: &str,
    luck_branch: &str,
    item: &LiuNianDto,
) -> Option<QuantAnnualLuckScore> {
    let (annual_stem, annual_branch) = split_gan_zhi(&item.gan_zhi)?;
    let stem_bundle = score_resolved_chart(
        day_master,
        &build_extended_inputs(
            natal_inputs,
            Some(luck_stem),
            None,
            Some(&annual_stem),
            Some(&annual_branch),
        ),
    );
    let branch_bundle = score_resolved_chart(
        day_master,
        &build_extended_inputs(
            natal_inputs,
            None,
            Some(luck_branch),
            Some(&annual_stem),
            Some(&annual_branch),
        ),
    );
    let first_half = item.index < 5;
    let active_bundle = if first_half {
        &stem_bundle
    } else {
        &branch_bundle
    };
    let annual_stem_pillar = resolve_pillar_score(&active_bundle.stem_scores, "流年干");
    let annual_branch_pillar = resolve_pillar_score(&active_bundle.branch_scores, "流年支");
    let luck_stem_score = resolve_pillar_score(&stem_bundle.stem_scores, "運干")
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let luck_branch_score = resolve_pillar_score(&branch_bundle.branch_scores, "運支")
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let annual_stem_score = annual_stem_pillar
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let annual_branch_score = annual_branch_pillar
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let annual_total_score = round1(annual_stem_score + annual_branch_score);
    let full_luck_score = round1((luck_stem_score + luck_branch_score) / 2.0);
    let active_luck_label = if first_half {
        format!("大運前五年：運干{}", luck_stem)
    } else {
        format!("大運後五年：運支{}", luck_branch)
    };
    let active_luck_score = if first_half {
        luck_stem_score
    } else {
        luck_branch_score
    };
    let combined_score = round1(active_luck_score + annual_total_score);
    let full_combined_score = round1(full_luck_score + annual_total_score);
    // R 值同樣計入原局因運、歲入局而變的分數（八字真理上冊 p70）。
    let natal_shift = natal_cells_total(active_bundle) - natal_cells_base;
    let impact_ratio = luck_impact_ratio(combined_score + natal_shift, natal_total);
    let full_impact_ratio = luck_impact_ratio(full_combined_score + natal_shift, natal_total);
    let effective_natal_score = luck_effective_natal_score(natal_total, impact_ratio);
    let full_effective_natal_score = luck_effective_natal_score(natal_total, full_impact_ratio);

    Some(QuantAnnualLuckScore {
        index: item.index,
        year: item.year,
        age: item.age,
        gan_zhi: item.gan_zhi.clone(),
        annual_stem_ten_god: item.stem_ten_god.clone(),
        annual_branch_hidden_stems: item.branch_hidden_stems.clone(),
        annual_branch_ten_gods: item.branch_ten_gods.clone(),
        active_luck_label,
        active_luck_score: round1(active_luck_score),
        full_luck_score,
        annual_stem_score: round1(annual_stem_score),
        annual_branch_score: round1(annual_branch_score),
        annual_total_score,
        combined_score,
        full_combined_score,
        impact_ratio,
        full_impact_ratio,
        effective_natal_score,
        full_effective_natal_score,
        tendency: tendency_label(combined_score, natal_total),
        full_tendency: tendency_label(full_combined_score, natal_total),
        overview_lines: Some(vec![
            format!("流年干：{:.1}", annual_stem_score),
            format!("流年支：{:.1}", annual_branch_score),
            format!("流年整柱：{:.1}", annual_total_score),
            format!("半運合參：{:.1}", combined_score),
            format!("整運合參（運干運支平均）：{:.1}", full_combined_score),
        ]),
        interaction_lines: Some(build_pillar_detail_lines(
            annual_stem_pillar,
            annual_branch_pillar,
        )),
        scoring_lines: Some(vec![
            format!("原局因運、歲入局而變：{:+.1}（已計入 R 值）", natal_shift),
            format!("R值（半運）：{:.2}", impact_ratio),
            format!("R值（整運）：{:.2}", full_impact_ratio),
            format!("作用後命局（半運）：{:.1}", effective_natal_score),
            format!("作用後命局（整運）：{:.1}", full_effective_natal_score),
        ]),
        details: format!(
            "原局 + 當值大運一字 + 流年干支同場裁決後，抽出流年干 {} 與流年支 {} 的列分。",
            annual_stem, annual_branch
        ),
    })
}

fn build_extended_inputs(
    natal_inputs: &CalculationInputs,
    luck_stem: Option<&str>,
    luck_branch: Option<&str>,
    annual_stem: Option<&str>,
    annual_branch: Option<&str>,
) -> CalculationInputs {
    let mut stems = natal_inputs.stems.clone();
    let mut branches = natal_inputs.branches.clone();
    if let Some(stem) = luck_stem {
        stems.push(StemInput {
            pillar: "運干".to_string(),
            stem: stem.to_string(),
            position_index: next_stem_position(&stems),
        });
    }
    if let Some(stem) = annual_stem {
        stems.push(StemInput {
            pillar: "流年干".to_string(),
            stem: stem.to_string(),
            position_index: next_stem_position(&stems),
        });
    }
    if let Some(branch) = luck_branch {
        branches.push(BranchInput {
            pillar: "運支".to_string(),
            branch: branch.to_string(),
            hidden_stems: hidden_stems_of(branch),
            position_index: branches.len(),
        });
    }
    if let Some(branch) = annual_branch {
        branches.push(BranchInput {
            pillar: "流年支".to_string(),
            branch: branch.to_string(),
            hidden_stems: hidden_stems_of(branch),
            position_index: branches.len(),
        });
    }
    CalculationInputs { stems, branches }
}

fn next_stem_position(stems: &[StemInput]) -> usize {
    stems
        .iter()
        .map(|item| item.position_index + 1)
        .max()
        .unwrap_or(0)
        .max(4)
}

fn resolve_pillar_score<'a>(
    scores: &'a [QuantModelPillarScore],
    pillar: &str,
) -> Option<&'a QuantModelPillarScore> {
    scores.iter().find(|score| score.pillar == pillar)
}

fn build_pillar_detail_lines(
    stem_score: Option<&QuantModelPillarScore>,
    branch_score: Option<&QuantModelPillarScore>,
) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(score) = stem_score {
        lines.push(format!("{}：{}", score.pillar, score.details));
    }
    if let Some(score) = branch_score {
        lines.push(format!("{}：{}", score.pillar, score.details));
    }
    lines
}

fn analyze_yong_shen(
    day_master: &str,
    total_score: f64,
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> QuantYongShen {
    let strong = total_score >= 0.0;
    let day_element = stem_element(day_master);
    let print_element = mother_of(&day_element);
    let peer_element = day_element.clone();
    let output_element = child_of(&day_element);
    let wealth_element = wealth_element_of(&day_element);
    let officer_element = officer_element_of(&day_element);
    let print_presence = resolve_print_presence(branch_scores, stem_scores);

    let (mut favorable, conditional, unfavorable, base_rule) = if !strong {
        match print_presence.as_str() {
            "天干見印" => (
                vec!["官殺", "印", "比劫"],
                vec![],
                vec!["食傷", "財"],
                "身弱且天干見印，用官印比，忌食財。",
            ),
            "印只在地支" => (
                vec!["印", "比劫"],
                vec!["官殺"],
                vec!["食傷", "財"],
                "身弱且印只在地支，用印比，忌食財；官可制的之。",
            ),
            _ => (
                vec!["印", "比劫"],
                vec![],
                vec!["食傷", "財", "官殺"],
                "身弱且局中無印，用印比，忌食財官。",
            ),
        }
    } else {
        match print_presence.as_str() {
            "天干見印" => (
                vec!["食傷", "財"],
                vec![],
                vec!["官殺", "印", "比劫"],
                "身強且天干見印，用食財，忌官印比。",
            ),
            "印只在地支" => (
                vec!["食傷", "財"],
                vec!["官殺"],
                vec!["印", "比劫"],
                "身強且印只在地支，用食財，忌印比；官可制的之。",
            ),
            _ => (
                vec!["食傷", "財", "官殺"],
                vec![],
                vec!["印", "比劫"],
                "身強且局中無印，用食財官，忌印比。",
            ),
        }
    };

    let family_scores = aggregate_family_scores(stem_scores, branch_scores);
    let element_scores = aggregate_element_scores(day_master, stem_scores, branch_scores);
    let mut caution = Vec::new();
    let mut caution_notes = Vec::new();

    if let Some(dominant_trouble) = find_dominant_trouble_family(&family_scores, &unfavorable) {
        let generator = generated_by_family(dominant_trouble);
        let dominant_magnitude = family_scores
            .get(dominant_trouble)
            .copied()
            .unwrap_or(0.0)
            .abs();
        let generator_magnitude = family_scores.get(generator).copied().unwrap_or(0.0).abs();
        if favorable.contains(&generator)
            && dominant_magnitude >= 35.0
            && dominant_magnitude >= generator_magnitude * 1.2
        {
            favorable.retain(|item| item != &generator);
            caution.push(generator);
            caution_notes.push(format!(
                "{}之氣偏重，{}易助其勢，故從可用中剔除。",
                family_label(dominant_trouble),
                family_label(generator)
            ));
        }
    }

    if !strong {
        if let Some(local_pressure_element) = find_dominant_pressure_element(&element_scores) {
            if let Some(generator) = family_by_element(
                &generated_by_element(local_pressure_element),
                &peer_element,
                &print_element,
                &output_element,
                &wealth_element,
                &officer_element,
            ) {
                if favorable.contains(&generator) {
                    favorable.retain(|item| item != &generator);
                    caution.push(generator);
                    caution_notes.push(format!(
                        "{}之氣在局部偏旺，{}會再生助該氣，宜改列為慎用。",
                        local_pressure_element,
                        family_label(generator)
                    ));
                }
            }
        }
    }

    let favorable_ten_gods = favorable
        .iter()
        .map(|item| family_label(item).to_string())
        .collect::<Vec<_>>();
    let conditional_ten_gods = conditional
        .iter()
        .map(|item| family_label(item).to_string())
        .collect::<Vec<_>>();
    let unfavorable_ten_gods = unfavorable
        .iter()
        .map(|item| family_label(item).to_string())
        .collect::<Vec<_>>();
    let caution_ten_gods = caution
        .iter()
        .map(|item| family_label(item).to_string())
        .collect::<Vec<_>>();
    let favorable_elements = family_elements(
        &favorable,
        &peer_element,
        &print_element,
        &output_element,
        &wealth_element,
        &officer_element,
    );
    let conditional_elements = family_elements(
        &conditional,
        &peer_element,
        &print_element,
        &output_element,
        &wealth_element,
        &officer_element,
    );
    let unfavorable_elements = family_elements(
        &unfavorable,
        &peer_element,
        &print_element,
        &output_element,
        &wealth_element,
        &officer_element,
    );
    let caution_elements = family_elements(
        &caution,
        &peer_element,
        &print_element,
        &output_element,
        &wealth_element,
        &officer_element,
    );

    let mut process = format!(
        "命局總分 {:.1}，先判為{}；{}；{}",
        total_score,
        if strong { "身強" } else { "身弱" },
        print_presence,
        base_rule
    );
    if !caution_notes.is_empty() {
        process.push_str(&format!(" 局部偏患修正：{}", caution_notes.join("；")));
    }

    let mut conclusion = format!(
        "依第三篇用神法，可用{}，忌{}",
        join_chinese(&favorable_elements),
        join_chinese(&unfavorable_elements)
    );
    if !conditional_elements.is_empty() {
        conclusion.push_str(&format!(
            "；{}可作輔助",
            join_chinese(&conditional_elements)
        ));
    }
    if !caution_elements.is_empty() {
        conclusion.push_str(&format!("；慎用{}", join_chinese(&caution_elements)));
    }

    QuantYongShen {
        method_summary: "第三篇用神：先定身強弱，再看印星位置，最後加局部偏患修正。".to_string(),
        strength_basis: if strong { "身強" } else { "身弱" }.to_string(),
        print_presence,
        favorable_ten_gods,
        favorable_elements,
        conditional_ten_gods,
        conditional_elements,
        unfavorable_ten_gods,
        unfavorable_elements,
        caution_ten_gods,
        caution_elements,
        process,
        conclusion,
    }
}

fn analyze_cong_pattern(
    total_score: f64,
    _strength_label: &str,
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> QuantCongPattern {
    const BOUNDARY_SCORE: f64 = 210.0;
    const TRUE_BOUNDARY_SCORE: f64 = 260.0;
    let (positive_totals, negative_totals) =
        aggregate_cong_family_scores(stem_scores, branch_scores);
    if total_score >= BOUNDARY_SCORE {
        let seal_peer = positive_totals.get("印").copied().unwrap_or(0.0)
            + positive_totals.get("比劫").copied().unwrap_or(0.0);
        let positive_total: f64 = positive_totals.values().sum();
        let seal_peer_dominant = seal_peer >= positive_total * 0.6;
        let true_follow = total_score >= TRUE_BOUNDARY_SCORE;
        let subtype = if seal_peer_dominant {
            "專旺印比"
        } else {
            "從強"
        };
        let dominant = if seal_peer_dominant {
            "印比".to_string()
        } else {
            dominant_map_key(&positive_totals)
                .unwrap_or("比劫")
                .to_string()
        };
        return QuantCongPattern {
            method_summary: "第十篇〈從格〉：先看命局是否跨過 ±210 臨界，再以 ±260 區分真從 / 假從，並依主導十神家族判別從強、專旺、從弱子類與可用神。".to_string(),
            boundary_score: BOUNDARY_SCORE,
            true_boundary_score: TRUE_BOUNDARY_SCORE,
            pattern: "從強".to_string(),
            authenticity: if seal_peer_dominant {
                if true_follow { "真專旺" } else { "假專旺" }
            } else if true_follow {
                "真從強"
            } else {
                "假從強"
            }
            .to_string(),
            subtype: subtype.to_string(),
            dominant_family: dominant,
            primary_use_gods: vec!["印比".to_string()],
            secondary_use_gods: if true_follow {
                vec!["食傷".to_string()]
            } else {
                Vec::new()
            },
            avoid_gods: if true_follow {
                vec!["官殺".to_string(), "財".to_string()]
            } else {
                vec!["食傷".to_string(), "官殺".to_string(), "財".to_string()]
            },
            risk_note: if true_follow {
                "真從強 / 真專旺較能承受小逆運，但仍以官殺運最忌；食傷可用，但仍不如印比。".to_string()
            } else {
                "假從強 / 假專旺最怕逆運破局，不能因總分剛跨過 +210 就直接把食傷當喜神。".to_string()
            },
            process: format!(
                "命局總分 {:.1}，高於 +210，已入從強臨界；正分主要集中在{}。{} +260，因此判作{}。",
                total_score,
                if seal_peer_dominant { "印比" } else { subtype },
                if true_follow { "又已超過" } else { "但尚未超過" },
                if seal_peer_dominant {
                    if true_follow { "真專旺" } else { "假專旺" }
                } else if true_follow {
                    "真從強"
                } else {
                    "假從強"
                }
            ),
            conclusion: if true_follow {
                format!("此局屬{}，主用印比，可兼用食傷，但食傷仍不如印比，且不應再用官殺。", if seal_peer_dominant { "真專旺" } else { "真從強" })
            } else {
                format!("此局屬{}，仍須以印比為主，不能把食傷視為正式可用神，亦較怕逆運破局。", if seal_peer_dominant { "假專旺" } else { "假從強" })
            },
        };
    }
    if total_score <= -BOUNDARY_SCORE {
        let dominant = dominant_map_key(&negative_totals).unwrap_or("食傷");
        let subtype = match dominant {
            "食傷" => "從兒格",
            "財" => "從財格",
            "官殺" => "從官殺格",
            _ => "從弱",
        };
        let true_follow = total_score <= -TRUE_BOUNDARY_SCORE;
        return QuantCongPattern {
            method_summary: "第十篇〈從格〉：先看命局是否跨過 ±210 臨界，再以 ±260 區分真從 / 假從，並依主導十神家族判別從強、專旺、從弱子類與可用神。".to_string(),
            boundary_score: BOUNDARY_SCORE,
            true_boundary_score: TRUE_BOUNDARY_SCORE,
            pattern: "從弱".to_string(),
            authenticity: if true_follow { "真從弱" } else { "假從弱" }.to_string(),
            subtype: subtype.to_string(),
            dominant_family: dominant.to_string(),
            primary_use_gods: vec!["食傷".to_string(), "財".to_string(), "官殺".to_string()],
            secondary_use_gods: Vec::new(),
            avoid_gods: vec!["印比".to_string()],
            risk_note: if true_follow {
                "真從弱較能承受小逆運，但仍以印比運最易破局。".to_string()
            } else {
                "假從弱最怕印比逆運，稍有扶身就可能由從格跌回正格。".to_string()
            },
            process: format!(
                "命局總分 {:.1}，低於 -210，已入從弱臨界；負分主要集中在{}。{} -260，因此判作{}。",
                total_score,
                dominant,
                if true_follow { "又已低過" } else { "但尚未低過" },
                if true_follow { "真從弱" } else { "假從弱" }
            ),
            conclusion: if true_follow {
                format!("此局屬真從弱，應順食財官之勢而行；其中以{}最主導，但仍忌印比逆扶。", subtype)
            } else {
                "此局屬假從弱，雖可順食財官之勢，但比真從弱更怕印比運破局。".to_string()
            },
        };
    }
    QuantCongPattern {
        method_summary: "第十篇〈從格〉：先看命局是否跨過 ±210 臨界，再以 ±260 區分真從 / 假從；未跨過 ±210 者，仍按正格論。".to_string(),
        boundary_score: BOUNDARY_SCORE,
        true_boundary_score: TRUE_BOUNDARY_SCORE,
        pattern: "不從".to_string(),
        authenticity: "不從".to_string(),
        subtype: if total_score >= 0.0 { "正格身強" } else { "正格身弱" }.to_string(),
        dominant_family: dominant_map_key(if total_score >= 0.0 { &positive_totals } else { &negative_totals })
            .unwrap_or(if total_score >= 0.0 { "比劫" } else { "食傷" })
            .to_string(),
        primary_use_gods: Vec::new(),
        secondary_use_gods: Vec::new(),
        avoid_gods: Vec::new(),
        risk_note: "此局未入從格臨界，仍應按正格身強 / 身弱與用神章法判斷。".to_string(),
        process: format!(
            "命局總分 {:.1}，介於 ±210 之間，未達從格臨界；即使局中有某一方特別強，也仍應按不從論。",
            total_score
        ),
        conclusion: format!("此局屬不從，應回到{}的正格判法。", if total_score >= 0.0 { "身強" } else { "身弱" }),
    }
}

fn analyze_special_pattern(
    normal_pattern_reference: &str,
    total_score: f64,
    bundle: &ResolvedScoreBundle,
    cong_pattern: &QuantCongPattern,
) -> QuantSpecialPattern {
    let mut rejection_reasons = Vec::new();
    if cong_pattern.pattern != "不從" {
        let primary_pattern = if cong_pattern.pattern == "從弱" {
            cong_pattern.subtype.clone()
        } else {
            cong_pattern.authenticity.clone()
        };
        return QuantSpecialPattern {
            primary_pattern: Some(primary_pattern.clone()),
            category: "從格/專旺".to_string(),
            authenticity: cong_pattern.authenticity.clone(),
            normal_pattern_reference: normal_pattern_reference.to_string(),
            special_use_gods: cong_pattern
                .primary_use_gods
                .iter()
                .chain(cong_pattern.secondary_use_gods.iter())
                .cloned()
                .collect(),
            special_avoid_gods: cong_pattern.avoid_gods.clone(),
            process: format!(
                "先以月令正格「{}」作參照；命局總分 {:.1}，{}，故特殊格局採{}。",
                normal_pattern_reference, total_score, cong_pattern.process, primary_pattern
            ),
            conclusion: format!(
                "特殊格成立：{}。正格用神保留參考，特殊格用神另以順勢為主。",
                primary_pattern
            ),
            rejection_reasons,
        };
    }

    rejection_reasons.push(format!("從格/專旺未成立：{}", cong_pattern.process));

    if let Some(pattern) =
        analyze_transformation_pattern(normal_pattern_reference, &bundle.resolved_state, bundle)
    {
        return pattern;
    }
    rejection_reasons.extend(transformation_rejection_reasons(&bundle.resolved_state));

    if let Some(pattern) = analyze_two_element_pattern(normal_pattern_reference, bundle) {
        return pattern;
    }
    rejection_reasons
        .push("兩神成象未成立：五行分布未達兩行高度集中，或其餘五行仍有破局力量。".to_string());

    QuantSpecialPattern {
        primary_pattern: None,
        category: "未成".to_string(),
        authenticity: "不成立".to_string(),
        normal_pattern_reference: normal_pattern_reference.to_string(),
        special_use_gods: Vec::new(),
        special_avoid_gods: Vec::new(),
        process: format!(
            "以月令正格「{}」作參照，依序檢查從格/專旺、化氣格、兩神成象，皆未達嚴格成格門檻。",
            normal_pattern_reference
        ),
        conclusion: "特殊格局未成，仍以正格與既有用神分析為主。".to_string(),
        rejection_reasons,
    }
}

fn transformation_rejection_reasons(state: &ResolvedChartState) -> Vec<String> {
    let mut reasons = Vec::new();
    let mut seen_pairs = BTreeSet::new();
    for effect in state.stem_combination_effects.values() {
        if !seen_pairs.insert(effect.pair.clone()) {
            continue;
        }
        if effect.transforms {
            reasons.push(format!(
                "化氣格未成立：{}已得化氣條件，但化神未達全局明顯占優門檻。",
                effect.pair
            ));
        } else {
            reasons.push(format!(
                "化氣格未成立：{}合而不化，不能判作{}化{}格。",
                effect.pair, effect.pair, effect.target_element
            ));
        }
    }
    if reasons.is_empty() {
        reasons.push("化氣格未成立：未見天干五合得月令或地支化局支援且化神明顯占優。".to_string());
    }
    reasons
}

fn analyze_transformation_pattern(
    normal_pattern_reference: &str,
    state: &ResolvedChartState,
    bundle: &ResolvedScoreBundle,
) -> Option<QuantSpecialPattern> {
    let element_scores =
        aggregate_element_magnitude_scores(&bundle.stem_scores, &bundle.branch_scores);
    let total_magnitude: f64 = element_scores.values().sum();
    if total_magnitude < 0.1 {
        return None;
    }

    let mut seen_pairs = BTreeSet::new();
    for effect in state.stem_combination_effects.values() {
        if !seen_pairs.insert(effect.pair.clone()) || !effect.transforms {
            continue;
        }
        let target_score = element_scores
            .get(&effect.target_element)
            .copied()
            .unwrap_or(0.0);
        let second_score = element_scores
            .iter()
            .filter(|(element, _)| *element != &effect.target_element)
            .map(|(_, value)| *value)
            .fold(0.0, f64::max);
        if target_score >= total_magnitude * 0.4 && target_score >= second_score * 1.25 {
            let pattern = format!("{}化{}格", effect.pair, effect.target_element);
            return Some(QuantSpecialPattern {
                primary_pattern: Some(pattern.clone()),
                category: "化氣格".to_string(),
                authenticity: "嚴格成立".to_string(),
                normal_pattern_reference: normal_pattern_reference.to_string(),
                special_use_gods: vec![effect.target_element.clone()],
                special_avoid_gods: elements_except(&[effect.target_element.as_str()]),
                process: format!(
                    "先以月令正格「{}」作參照；{}，且化神{}占全局量化力 {:.1}% ，高於次強五行，故按嚴格化氣格成立。",
                    normal_pattern_reference,
                    effect.note,
                    effect.target_element,
                    target_score / total_magnitude * 100.0
                ),
                conclusion: format!(
                    "特殊格成立：{}。特殊格用神以化神{}為主，正格用神另作參考。",
                    pattern, effect.target_element
                ),
                rejection_reasons: Vec::new(),
            });
        }
    }

    None
}

fn analyze_two_element_pattern(
    normal_pattern_reference: &str,
    bundle: &ResolvedScoreBundle,
) -> Option<QuantSpecialPattern> {
    let mut element_scores =
        aggregate_element_magnitude_scores(&bundle.stem_scores, &bundle.branch_scores)
            .into_iter()
            .collect::<Vec<_>>();
    element_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    let total: f64 = element_scores.iter().map(|(_, value)| *value).sum();
    if total < 0.1 || element_scores.len() < 2 {
        return None;
    }

    let (first, first_score) = &element_scores[0];
    let (second, second_score) = &element_scores[1];
    let third_score = element_scores
        .get(2)
        .map(|(_, value)| *value)
        .unwrap_or(0.0);
    let top_two_ratio = (first_score + second_score) / total;
    let third_ratio = third_score / total;
    if top_two_ratio < 0.82 || third_ratio > 0.15 {
        return None;
    }

    let ordered = if generates_element(first, second) {
        (first.clone(), second.clone())
    } else if generates_element(second, first) {
        (second.clone(), first.clone())
    } else {
        return None;
    };
    let pattern = format!("{}{}成象", ordered.0, ordered.1);

    Some(QuantSpecialPattern {
        primary_pattern: Some(pattern.clone()),
        category: "兩神成象".to_string(),
        authenticity: "嚴格成立".to_string(),
        normal_pattern_reference: normal_pattern_reference.to_string(),
        special_use_gods: vec![ordered.0.clone(), ordered.1.clone()],
        special_avoid_gods: elements_except(&[ordered.0.as_str(), ordered.1.as_str()]),
        process: format!(
            "先以月令正格「{}」作參照；{}、{}合計占全局量化力 {:.1}% ，第三五行僅 {:.1}% ，且兩行相生，故按兩神成象成立。",
            normal_pattern_reference,
            first,
            second,
            top_two_ratio * 100.0,
            third_ratio * 100.0
        ),
        conclusion: format!(
            "特殊格成立：{}。特殊格用神以{}、{}兩行流通為主，正格用神另作參考。",
            pattern, ordered.0, ordered.1
        ),
        rejection_reasons: Vec::new(),
    })
}

fn analyze_sha_yin(
    day_master: &str,
    total_score: f64,
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> QuantShaYin {
    let day_element = stem_element(day_master);
    let stem_pattern = resolve_stem_sha_yin_pattern(&day_element, total_score, stem_scores);
    let branch_pattern = resolve_branch_sha_yin_pattern(&day_element, total_score, branch_scores);
    QuantShaYin {
        method_summary:
            "第九篇殺印相生：先看年干殺、月干印、日元是否成鏈，再看印能否承受殺；地支只接受純氣鏈。"
                .to_string(),
        stem_pattern_found: stem_pattern.found,
        stem_transformed: stem_pattern.transformed,
        stem_chain: stem_pattern.chain.clone(),
        stem_source_negative_score: stem_pattern.source_negative_score,
        stem_seal_support_score: stem_pattern.seal_support_score,
        stem_adjusted_total_score: stem_pattern.adjusted_total_score,
        branch_pattern_found: branch_pattern.found,
        branch_transformed: branch_pattern.transformed,
        branch_chain: branch_pattern.chain.clone(),
        branch_source_negative_score: branch_pattern.source_negative_score,
        branch_seal_support_score: branch_pattern.seal_support_score,
        branch_adjusted_total_score: branch_pattern.adjusted_total_score,
        process: build_sha_yin_process(total_score, &stem_pattern, &branch_pattern),
        conclusion: build_sha_yin_conclusion(&stem_pattern, &branch_pattern),
    }
}

#[derive(Debug, Clone)]
struct ShaYinPattern {
    found: bool,
    transformed: bool,
    chain: Option<String>,
    source_negative_score: Option<f64>,
    seal_support_score: Option<f64>,
    adjusted_total_score: Option<f64>,
}

fn resolve_stem_sha_yin_pattern(
    day_element: &str,
    total_score: f64,
    stem_scores: &[QuantModelPillarScore],
) -> ShaYinPattern {
    let year = stem_scores.iter().find(|item| item.pillar == "年干");
    let month = stem_scores.iter().find(|item| item.pillar == "月干");
    let (Some(year), Some(month)) = (year, month) else {
        return ShaYinPattern {
            found: false,
            transformed: false,
            chain: Some("天干殺印相生：缺少年干或月干資料。".to_string()),
            source_negative_score: None,
            seal_support_score: None,
            adjusted_total_score: None,
        };
    };
    let year_element = stem_element(&year.target);
    let month_element = stem_element(&month.target);
    let chain = format!(
        "年干{}({}) -> 月干{}({}) -> 日元{}",
        year.target, year.ten_god, month.target, month.ten_god, day_element
    );
    let valid = is_kill_god(&year.ten_god)
        && is_seal_god(&month.ten_god)
        && generates(&year_element, &month_element)
        && generates(&month_element, day_element)
        && year.final_score < 0.0
        && month.final_score > 0.0;
    if !valid {
        return ShaYinPattern {
            found: false,
            transformed: false,
            chain: Some(format!(
                "{}，不符合年干殺、月干印、月印生日元的鏈條。",
                chain
            )),
            source_negative_score: None,
            seal_support_score: None,
            adjusted_total_score: None,
        };
    }
    let total_seal_support: f64 = stem_scores
        .iter()
        .filter(|item| is_seal_god(&item.ten_god) && item.final_score > 0.0)
        .map(|item| item.final_score)
        .sum();
    let transformed = total_seal_support >= year.final_score.abs();
    ShaYinPattern {
        found: true,
        transformed,
        chain: Some(chain),
        source_negative_score: Some(round1(year.final_score)),
        seal_support_score: Some(round1(total_seal_support)),
        adjusted_total_score: Some(if transformed {
            round1(total_score + year.final_score.abs() * 2.0)
        } else {
            total_score
        }),
    }
}

fn resolve_branch_sha_yin_pattern(
    day_element: &str,
    total_score: f64,
    branch_scores: &[QuantModelPillarScore],
) -> ShaYinPattern {
    let year = branch_scores.iter().find(|item| item.pillar == "年支");
    let month = branch_scores.iter().find(|item| item.pillar == "月支");
    let (Some(year), Some(month)) = (year, month) else {
        return ShaYinPattern {
            found: false,
            transformed: false,
            chain: Some("地支殺印相生：缺少年支或月支資料。".to_string()),
            source_negative_score: None,
            seal_support_score: None,
            adjusted_total_score: None,
        };
    };
    let (chain, valid_chain) = if year.target == "酉" && month.target == "子" {
        (
            "年支酉 -> 月支子 -> 木日元".to_string(),
            day_element == "木",
        )
    } else if year.target == "子" && month.target == "卯" {
        (
            "年支子 -> 月支卯 -> 火日元".to_string(),
            day_element == "火",
        )
    } else {
        (
            "地支鏈不屬於第九篇允許的純氣殺印相生型。".to_string(),
            false,
        )
    };
    if !valid_chain || year.final_score >= 0.0 || month.final_score <= 0.0 {
        return ShaYinPattern {
            found: false,
            transformed: false,
            chain: Some(chain),
            source_negative_score: None,
            seal_support_score: None,
            adjusted_total_score: None,
        };
    }
    let transformed = month.final_score >= year.final_score.abs();
    ShaYinPattern {
        found: true,
        transformed,
        chain: Some(chain),
        source_negative_score: Some(round1(year.final_score)),
        seal_support_score: Some(round1(month.final_score)),
        adjusted_total_score: Some(if transformed {
            round1(total_score + year.final_score.abs() * 2.0)
        } else {
            total_score
        }),
    }
}

fn build_sha_yin_process(
    total_score: f64,
    stem_pattern: &ShaYinPattern,
    branch_pattern: &ShaYinPattern,
) -> String {
    let mut builder = format!("原局總分 {:.1}。", total_score);
    if let Some(chain) = &stem_pattern.chain {
        builder.push_str(" 天干鏈：");
        builder.push_str(chain);
        if stem_pattern.found {
            builder.push_str(&format!(
                "；殺分 {}，印支援 {}。{}",
                format_optional_score(stem_pattern.source_negative_score),
                format_optional_score(stem_pattern.seal_support_score),
                if stem_pattern.transformed {
                    "印能承殺，故年干殺可負能轉正。"
                } else {
                    "印力不足以承殺，故天干殺印相生不成立。"
                }
            ));
        }
    }
    if let Some(chain) = &branch_pattern.chain {
        builder.push_str(" 地支鏈：");
        builder.push_str(chain);
        if branch_pattern.found {
            builder.push_str(&format!(
                "；殺分 {}，印支援 {}。{}",
                format_optional_score(branch_pattern.source_negative_score),
                format_optional_score(branch_pattern.seal_support_score),
                if branch_pattern.transformed {
                    "月支印能承受年支殺，地支負能可轉正。"
                } else {
                    "月支印力不足，地支負能轉正不成立。"
                }
            ));
        }
    }
    builder
}

fn build_sha_yin_conclusion(
    stem_pattern: &ShaYinPattern,
    branch_pattern: &ShaYinPattern,
) -> String {
    if stem_pattern.transformed && branch_pattern.transformed {
        "天干與地支兩路殺印相生皆成立，命局負能可雙重轉正。".to_string()
    } else if stem_pattern.transformed {
        "天干殺印相生成立，可把年干殺的負能轉正。".to_string()
    } else if branch_pattern.transformed {
        "地支殺印相生成立，可把年支殺的負能轉正。".to_string()
    } else if stem_pattern.found || branch_pattern.found {
        "命局雖有殺印相生外形，但印力不足或鏈條不純，負能轉正不成立。".to_string()
    } else {
        "命局未形成第九篇所述的殺印相生鏈條。".to_string()
    }
}

fn build_natal_interactions(
    state: &ResolvedChartState,
    branch_scores: &[QuantModelPillarScore],
) -> Vec<QuantModelInteraction> {
    let mut rows = Vec::new();
    add_stem_combination_rows(&mut rows, state);
    add_three_meeting_rows(&mut rows, state);
    add_three_combination_rows(&mut rows, state);
    add_half_combination_rows(&mut rows, state);
    add_six_combination_rows(&mut rows, state);
    add_static_six_combination_rows(&mut rows, &state.branches);
    add_stem_clash_rows(&mut rows, &state.stems, &state.stem_clash_effects);
    add_branch_clash_rows(&mut rows, state, branch_scores);
    add_informational_pair_rows(&mut rows, &state.branches);
    add_self_punishment_rows(&mut rows, &state.branches);
    dedupe_and_sort_interactions(rows)
}

fn add_stem_combination_rows(rows: &mut Vec<QuantModelInteraction>, state: &ResolvedChartState) {
    let mut grouped_indexes = HashMap::<String, Vec<usize>>::new();
    for (index, effect) in &state.stem_combination_effects {
        grouped_indexes
            .entry(effect.combine_note.clone())
            .or_default()
            .push(*index);
    }
    for indexes in grouped_indexes.values_mut() {
        indexes.sort_unstable();
        indexes.dedup();
        if indexes.is_empty() {
            continue;
        }
        let Some(effect) = state.stem_combination_effects.get(&indexes[0]) else {
            continue;
        };
        rows.push(QuantModelInteraction {
            scope: "天干".to_string(),
            r#type: "合".to_string(),
            target: join_stems(&state.stems, indexes),
            outcome: if effect.transforms {
                format!("化{}", effect.target_element)
            } else {
                format!("{}而不化", effect.target_element)
            },
            pillars: join_stem_pillars(&state.stems, indexes),
            detail: format!("量化模型裁決為天干{}", effect.note),
        });
    }
}

fn add_three_meeting_rows(rows: &mut Vec<QuantModelInteraction>, state: &ResolvedChartState) {
    for candidate in &state.three_meeting_candidates {
        let indexes = [
            candidate.first_index,
            candidate.second_index,
            candidate.third_index,
        ];
        let active_transform =
            has_category(state, &BranchTransformCategory::ThreeMeeting, &indexes);
        if candidate.transforms && !active_transform {
            continue;
        }
        if !candidate.transforms && has_any_transform(state, &indexes) {
            continue;
        }
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: if candidate.transforms {
                "三會化".to_string()
            } else {
                "三會".to_string()
            },
            target: join_branches(&state.branches, &indexes),
            outcome: if candidate.transforms {
                format!("化{}", candidate.element)
            } else {
                "-".to_string()
            },
            pillars: join_pillars(&state.branches, &indexes),
            detail: if candidate.transforms {
                format!("量化模型裁決為地支三會化{}", candidate.element)
            } else {
                format!(
                    "量化模型偵測到地支三會{}局，但未達合化條件",
                    candidate.element
                )
            },
        });
    }
}

fn add_three_combination_rows(rows: &mut Vec<QuantModelInteraction>, state: &ResolvedChartState) {
    for candidate in &state.three_combination_candidates {
        let indexes = [
            candidate.first_index,
            candidate.second_index,
            candidate.third_index,
        ];
        let active_transform =
            has_category(state, &BranchTransformCategory::ThreeCombination, &indexes);
        if candidate.transforms && !active_transform {
            continue;
        }
        if !candidate.transforms && has_any_transform(state, &indexes) {
            continue;
        }
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: if candidate.transforms {
                "三合化".to_string()
            } else {
                "三合".to_string()
            },
            target: join_branches(&state.branches, &indexes),
            outcome: if candidate.transforms {
                format!("化{}", candidate.element)
            } else {
                "-".to_string()
            },
            pillars: join_pillars(&state.branches, &indexes),
            detail: if candidate.transforms {
                format!("量化模型裁決為地支三合化{}", candidate.element)
            } else {
                format!(
                    "量化模型偵測到地支三合{}局，但未達合化條件",
                    candidate.element
                )
            },
        });
    }
}

fn add_half_combination_rows(rows: &mut Vec<QuantModelInteraction>, state: &ResolvedChartState) {
    for candidate in &state.half_combination_candidates {
        let indexes = [candidate.left_index, candidate.right_index];
        let active_transform =
            has_category(state, &BranchTransformCategory::HalfCombination, &indexes);
        if candidate.transforms && !active_transform {
            continue;
        }
        if !candidate.transforms
            && (has_any_transform(state, &indexes)
                || state
                    .six_combination_effects
                    .contains_key(&candidate.left_index)
                || state
                    .six_combination_effects
                    .contains_key(&candidate.right_index)
                || state
                    .branch_prune_effects
                    .contains_key(&candidate.left_index)
                || state
                    .branch_prune_effects
                    .contains_key(&candidate.right_index))
        {
            continue;
        }
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: if candidate.transforms {
                "半合化".to_string()
            } else {
                "半合".to_string()
            },
            target: join_pair_branches(
                &state.branches,
                candidate.left_index,
                candidate.right_index,
            ),
            outcome: if candidate.transforms {
                format!("化{}", candidate.element)
            } else {
                "不化".to_string()
            },
            pillars: join_pair_pillars(
                &state.branches,
                candidate.left_index,
                candidate.right_index,
            ),
            detail: if candidate.transforms {
                format!("量化模型裁決為地支半合化{}", candidate.element)
            } else {
                format!(
                    "量化模型偵測到地支半合{}，但未達合化條件",
                    candidate.element
                )
            },
        });
    }
}

fn add_six_combination_rows(rows: &mut Vec<QuantModelInteraction>, state: &ResolvedChartState) {
    let mut transformed_groups = HashMap::<String, Vec<usize>>::new();
    for (index, transform) in &state.branch_transforms {
        if transform.category != BranchTransformCategory::SixCombination {
            continue;
        }
        transformed_groups
            .entry(transform.combine_note.clone())
            .or_default()
            .push(*index);
    }
    for indexes in transformed_groups.values_mut() {
        indexes.sort_unstable();
        indexes.dedup();
        let Some(transform) = state.branch_transforms.get(&indexes[0]) else {
            continue;
        };
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: "合".to_string(),
            target: join_branches(&state.branches, &display_indexes(indexes)),
            outcome: format!("化{}", transform.element),
            pillars: join_pillars(&state.branches, &display_indexes(indexes)),
            detail: format!("量化模型裁決為地支六合化{}", transform.element),
        });
    }

    let mut non_transform_groups = HashMap::<String, Vec<usize>>::new();
    for (index, effect) in &state.six_combination_effects {
        non_transform_groups
            .entry(effect.pair.clone())
            .or_default()
            .extend([*index, effect.other_index]);
    }
    for indexes in non_transform_groups.values_mut() {
        indexes.sort_unstable();
        indexes.dedup();
        if indexes.is_empty() {
            continue;
        }
        let Some(effect) = state.six_combination_effects.get(&indexes[0]) else {
            continue;
        };
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: "合".to_string(),
            target: join_branches(&state.branches, &display_indexes(indexes)),
            outcome: format!("{}而不化", effect.element),
            pillars: join_pillars(&state.branches, &display_indexes(indexes)),
            detail: format!(
                "量化模型裁決為地支六合{}而不化，保留餘力 {}",
                effect.element,
                round3_string(effect.remainder)
            ),
        });
    }
}

fn add_static_six_combination_rows(
    rows: &mut Vec<QuantModelInteraction>,
    branches: &[BranchInput],
) {
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            if !is_six_combination(&left.branch, &right.branch) {
                continue;
            }
            let element = resolve_six_combination_element(&left.branch, &right.branch);
            rows.push(QuantModelInteraction {
                scope: "地支".to_string(),
                r#type: "合".to_string(),
                target: join_pair_branches(branches, left.position_index, right.position_index),
                outcome: "-".to_string(),
                pillars: join_pair_pillars(branches, left.position_index, right.position_index),
                detail: format!("地支六合，對應{}", element),
            });
        }
    }
}

fn add_stem_clash_rows(
    rows: &mut Vec<QuantModelInteraction>,
    stems: &[StemInput],
    clash_effects: &HashMap<usize, StemClashEffect>,
) {
    for i in 0..stems.len() {
        for j in (i + 1)..stems.len() {
            let left = &stems[i];
            let right = &stems[j];
            if !is_stem_clash(&left.stem, &right.stem)
                || !clash_effects.contains_key(&left.position_index)
                || !clash_effects.contains_key(&right.position_index)
            {
                continue;
            }
            let dist = stem_pair_distance(left, right);
            if !is_supported_stem_clash_distance(dist) {
                continue;
            }
            let remainder = stem_clash_remainder(dist);
            let note = match dist {
                1 => format!("{}{}緊貼相沖", left.stem, right.stem),
                2 => format!("{}{}隔一位相沖", left.stem, right.stem),
                _ => format!("{}{}隔二位相沖", left.stem, right.stem),
            };
            rows.push(QuantModelInteraction {
                scope: "天干".to_string(),
                r#type: "沖".to_string(),
                target: format!("{}{}", left.stem, right.stem),
                outcome: "-".to_string(),
                pillars: join_stem_pillars(stems, &[left.position_index, right.position_index]),
                detail: format!(
                    "量化模型裁決為天干{}，保留餘力 {}",
                    note,
                    round3_string(remainder)
                ),
            });
        }
    }
}

fn add_branch_clash_rows(
    rows: &mut Vec<QuantModelInteraction>,
    state: &ResolvedChartState,
    branch_scores: &[QuantModelPillarScore],
) {
    let mut details = HashMap::<String, String>::new();
    for score in branch_scores {
        let Some(combine_note) = &score.combine_note else {
            continue;
        };
        let Some(target) = combine_note.strip_prefix("沖@") else {
            continue;
        };
        let Some(left) = resolve_branch_by_pillar(&state.branches, &score.pillar) else {
            continue;
        };
        let Some(right) = resolve_branch_by_target(&state.branches, target) else {
            continue;
        };
        details
            .entry(pair_key(left.position_index, right.position_index))
            .or_insert_with(|| {
                resolve_clash_detail(state, left.position_index, right.position_index, score)
            });
    }
    for (key, detail) in details {
        let (left_index, right_index) = parse_pair_key(&key);
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: "沖".to_string(),
            target: join_pair_branches(&state.branches, left_index, right_index),
            outcome: "-".to_string(),
            pillars: join_pair_pillars(&state.branches, left_index, right_index),
            detail,
        });
    }
}

fn add_informational_pair_rows(rows: &mut Vec<QuantModelInteraction>, branches: &[BranchInput]) {
    const INFO_PAIR_RULES: [(&str, &str, &str, &str); 13] = [
        ("子", "卯", "刑", "子卯相刑(無禮之刑)"),
        ("寅", "巳", "刑", "寅巳申三刑(恃勢之刑)"),
        ("巳", "申", "刑", "寅巳申三刑(恃勢之刑)"),
        ("寅", "申", "刑", "寅巳申三刑(恃勢之刑)"),
        ("丑", "戌", "刑", "丑戌未三刑(無恩之刑)"),
        ("戌", "未", "刑", "丑戌未三刑(無恩之刑)"),
        ("丑", "未", "刑", "丑戌未三刑(無恩之刑)"),
        ("子", "酉", "破", "地支相破"),
        ("卯", "午", "破", "地支相破"),
        ("辰", "丑", "破", "地支相破"),
        ("未", "戌", "破", "地支相破"),
        ("寅", "亥", "破", "地支相破"),
        ("巳", "申", "破", "地支相破"),
    ];

    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            for rule in INFO_PAIR_RULES {
                if matches_pair(&branches[i].branch, &branches[j].branch, rule.0, rule.1) {
                    rows.push(QuantModelInteraction {
                        scope: "地支".to_string(),
                        r#type: rule.2.to_string(),
                        target: join_pair_branches(
                            branches,
                            branches[i].position_index,
                            branches[j].position_index,
                        ),
                        outcome: "-".to_string(),
                        pillars: join_pair_pillars(
                            branches,
                            branches[i].position_index,
                            branches[j].position_index,
                        ),
                        detail: rule.3.to_string(),
                    });
                }
            }
        }
    }
}

fn add_self_punishment_rows(rows: &mut Vec<QuantModelInteraction>, branches: &[BranchInput]) {
    let mut grouped = HashMap::<String, Vec<usize>>::new();
    for branch in branches {
        grouped
            .entry(branch.branch.clone())
            .or_default()
            .push(branch.position_index);
    }
    for branch in ["辰", "午", "酉", "亥"] {
        let Some(indexes) = grouped.get(branch) else {
            continue;
        };
        if indexes.len() < 2 {
            continue;
        }
        rows.push(QuantModelInteraction {
            scope: "地支".to_string(),
            r#type: "刑".to_string(),
            target: format!("{}{}", branch, branch),
            outcome: "-".to_string(),
            pillars: join_pillars(branches, indexes),
            detail: "自刑".to_string(),
        });
    }
}

fn has_category(
    state: &ResolvedChartState,
    category: &BranchTransformCategory,
    indexes: &[usize],
) -> bool {
    indexes.iter().all(|index| {
        state
            .branch_transforms
            .get(index)
            .is_some_and(|t| &t.category == category)
    })
}

fn has_any_transform(state: &ResolvedChartState, indexes: &[usize]) -> bool {
    indexes
        .iter()
        .any(|index| state.branch_transforms.contains_key(index))
}

fn resolve_branch_by_pillar<'a>(
    branches: &'a [BranchInput],
    pillar: &str,
) -> Option<&'a BranchInput> {
    branches.iter().find(|branch| branch.pillar == pillar)
}

fn resolve_branch_by_target<'a>(
    branches: &'a [BranchInput],
    target: &str,
) -> Option<&'a BranchInput> {
    branches
        .iter()
        .find(|branch| format!("{}{}", branch.pillar, branch.branch) == target)
}

fn resolve_clash_detail(
    state: &ResolvedChartState,
    left_index: usize,
    right_index: usize,
    score: &QuantModelPillarScore,
) -> String {
    if let Some(effect) = state.branch_prune_effects.get(&left_index) {
        if !effect.note.is_empty() {
            return effect.note.clone();
        }
    }
    if let Some(effect) = state.branch_prune_effects.get(&right_index) {
        if !effect.note.is_empty() {
            return effect.note.clone();
        }
    }
    if let Some(rows) = &score.rows {
        for row in rows {
            if row
                .interaction
                .as_ref()
                .is_some_and(|interaction| interaction.contains('沖'))
            {
                if let Some(note) = &row.note {
                    if !note.is_empty() {
                        return note.clone();
                    }
                }
            }
        }
    }
    if !score.details.is_empty() {
        return score.details.clone();
    }
    "地支六沖".to_string()
}

fn join_branches(branches: &[BranchInput], indexes: &[usize]) -> String {
    indexes
        .iter()
        .filter_map(|index| resolve_branch_by_index(branches, *index))
        .map(|branch| branch.branch.clone())
        .collect::<Vec<_>>()
        .join("")
}

fn join_pair_branches(branches: &[BranchInput], left_index: usize, right_index: usize) -> String {
    join_branches(branches, &display_indexes(&[left_index, right_index]))
}

fn join_stems(stems: &[StemInput], indexes: &[usize]) -> String {
    indexes
        .iter()
        .filter_map(|index| resolve_stem_by_index(stems, *index))
        .map(|stem| stem.stem.clone())
        .collect::<Vec<_>>()
        .join("")
}

fn join_pillars(branches: &[BranchInput], indexes: &[usize]) -> String {
    indexes
        .iter()
        .filter_map(|index| resolve_branch_by_index(branches, *index))
        .map(|branch| format!("{}({})", to_display_pillar(&branch.pillar), branch.branch))
        .collect::<Vec<_>>()
        .join("、")
}

fn join_pair_pillars(branches: &[BranchInput], left_index: usize, right_index: usize) -> String {
    join_pillars(branches, &display_indexes(&[left_index, right_index]))
}

fn join_stem_pillars(stems: &[StemInput], indexes: &[usize]) -> String {
    indexes
        .iter()
        .filter_map(|index| resolve_stem_by_index(stems, *index))
        .map(|stem| format!("{}({})", to_display_pillar(&stem.pillar), stem.stem))
        .collect::<Vec<_>>()
        .join("、")
}

fn resolve_branch_by_index(branches: &[BranchInput], index: usize) -> Option<&BranchInput> {
    branches
        .iter()
        .find(|branch| branch.position_index == index)
}

fn resolve_stem_by_index(stems: &[StemInput], index: usize) -> Option<&StemInput> {
    stems.iter().find(|stem| stem.position_index == index)
}

fn display_indexes(indexes: &[usize]) -> Vec<usize> {
    let mut display = indexes.to_vec();
    if display.len() == 2 {
        display.sort_unstable_by(|a, b| b.cmp(a));
    }
    display
}

fn to_display_pillar(pillar: &str) -> String {
    pillar.replace('干', "柱").replace('支', "柱")
}

fn pair_key(left_index: usize, right_index: usize) -> String {
    format!(
        "{}-{}",
        left_index.min(right_index),
        left_index.max(right_index)
    )
}

fn parse_pair_key(key: &str) -> (usize, usize) {
    let mut parts = key.split('-');
    let left = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let right = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    (left, right)
}

fn matches_pair(left: &str, right: &str, expected_left: &str, expected_right: &str) -> bool {
    (left == expected_left && right == expected_right)
        || (left == expected_right && right == expected_left)
}

fn dedupe_and_sort_interactions(rows: Vec<QuantModelInteraction>) -> Vec<QuantModelInteraction> {
    let mut unique = HashMap::<String, QuantModelInteraction>::new();
    for row in rows {
        let key = format!(
            "{}|{}|{}|{}",
            row.scope, row.r#type, row.target, row.pillars
        );
        unique.entry(key).or_insert(row);
    }
    let mut values = unique.into_values().collect::<Vec<_>>();
    values.sort_by(|a, b| {
        interaction_type_priority(&a.r#type)
            .cmp(&interaction_type_priority(&b.r#type))
            .then_with(|| {
                interaction_scope_priority(&a.scope).cmp(&interaction_scope_priority(&b.scope))
            })
            .then_with(|| a.target.cmp(&b.target))
    });
    values
}

fn interaction_scope_priority(scope: &str) -> i32 {
    match scope {
        "天干" => 1,
        "地支" => 2,
        _ => 99,
    }
}

fn interaction_type_priority(kind: &str) -> i32 {
    match kind {
        "三會化" => 1,
        "三合化" => 2,
        "三會" => 3,
        "三合" => 4,
        "半合化" => 5,
        "半合" => 6,
        "合" => 7,
        "沖" => 8,
        "刑" => 9,
        "破" => 10,
        _ => 99,
    }
}

fn resolve_three_meeting_candidates(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> Vec<ThreeMeetingCandidate> {
    let mut candidates = Vec::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            for k in (j + 1)..branches.len() {
                if let Some((element, label)) = resolve_three_meeting(
                    &branches[i].branch,
                    &branches[j].branch,
                    &branches[k].branch,
                ) {
                    // 三會成化力量 5.25，月令剋制不住，只要天干見化神即可化（上冊 p133）。
                    let transforms = has_visible_transforming_stem(&element, stems, day_master);
                    candidates.push(ThreeMeetingCandidate {
                        first_index: branches[i].position_index,
                        second_index: branches[j].position_index,
                        third_index: branches[k].position_index,
                        element,
                        transforms,
                        label,
                    });
                }
            }
        }
    }
    candidates
}

fn resolve_three_combination_candidates(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> Vec<ThreeCombinationCandidate> {
    let mut candidates = Vec::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            for k in (j + 1)..branches.len() {
                if let Some((element, label)) = resolve_three_combination(
                    &branches[i].branch,
                    &branches[j].branch,
                    &branches[k].branch,
                ) {
                    let transforms = should_transform_branch_combination(
                        &element,
                        branches,
                        &[
                            branches[i].position_index,
                            branches[j].position_index,
                            branches[k].position_index,
                        ],
                        stems,
                        day_master,
                    );
                    candidates.push(ThreeCombinationCandidate {
                        first_index: branches[i].position_index,
                        second_index: branches[j].position_index,
                        third_index: branches[k].position_index,
                        element,
                        transforms,
                        label,
                    });
                }
            }
        }
    }
    candidates
}

fn resolve_half_combination_candidates(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> Vec<HalfCombinationCandidate> {
    let mut candidates = Vec::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let pair_distance = branch_pair_distance(&branches[i], &branches[j]);
            if let Some((element, note, combine_note)) =
                resolve_half_combination(&branches[i].branch, &branches[j].branch)
            {
                let transforms = should_transform_branch_combination(
                    &element,
                    branches,
                    &[branches[i].position_index, branches[j].position_index],
                    stems,
                    day_master,
                );
                candidates.push(HalfCombinationCandidate {
                    left_index: branches[i].position_index,
                    right_index: branches[j].position_index,
                    element,
                    distance: pair_distance,
                    transforms,
                    note,
                    combine_note,
                });
            }
        }
    }
    candidates
}

/// 四庫匯聚：原局四支為辰戌丑未各一（八字真理上冊 p55）。
fn four_storage_indexes(branches: &[BranchInput]) -> Option<Vec<usize>> {
    let natal: Vec<&BranchInput> = branches
        .iter()
        .filter(|item| !is_floating_pillar(&item.pillar))
        .collect();
    let mut names: Vec<&str> = natal.iter().map(|item| item.branch.as_str()).collect();
    names.sort();
    let mut expected = vec!["辰", "戌", "丑", "未"];
    expected.sort();
    (natal.len() == 4 && names == expected)
        .then(|| natal.iter().map(|item| item.position_index).collect())
}

/// 四庫匯聚，天干見土即合化巨土：各支 1.15 個土，旁氣全被化掉，沒有餘氣（上冊 p55）。
fn resolve_four_storage_transforms(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    let Some(indexes) = four_storage_indexes(branches) else {
        return transforms;
    };
    if !has_visible_transforming_stem("土", stems, day_master) {
        return transforms;
    }
    for index in indexes {
        let branch = branches
            .iter()
            .find(|item| item.position_index == index)
            .expect("four storage branch");
        transforms.insert(
            index,
            create_branch_transform(
                BranchTransformCategory::ThreeMeeting,
                CombinationBranchRole::Specialist,
                "土",
                "四庫匯聚化土",
                "四庫匯聚化土@meeting",
                1.15,
                0.0,
                branch,
            ),
        );
    }
    transforms
}

fn resolve_three_meeting_transforms(
    candidates: &[ThreeMeetingCandidate],
    branches: &[BranchInput],
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    for candidate in candidates.iter().filter(|item| item.transforms) {
        for index in [
            candidate.first_index,
            candidate.second_index,
            candidate.third_index,
        ] {
            let branch = &branches[index];
            transforms.insert(
                index,
                create_branch_transform(
                    BranchTransformCategory::ThreeMeeting,
                    branch_role(&candidate.label, &branch.branch),
                    &candidate.element,
                    &format!("三會化{}", candidate.element),
                    &format!("三會化{}@meeting", candidate.element),
                    resolve_three_meeting_factor(
                        &candidate.label,
                        branch_role(&candidate.label, &branch.branch),
                    ),
                    resolve_three_meeting_residual_factor(
                        &candidate.label,
                        branch_role(&candidate.label, &branch.branch),
                    ),
                    branch,
                ),
            );
        }
    }
    transforms
}

fn resolve_three_combination_transforms(
    candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    branches: &[BranchInput],
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    for candidate in candidates.iter().filter(|item| item.transforms) {
        if three_meeting_candidates.iter().any(|meeting| {
            meeting.involves(candidate.first_index)
                && meeting.involves(candidate.second_index)
                && meeting.involves(candidate.third_index)
        }) {
            continue;
        }
        for index in [
            candidate.first_index,
            candidate.second_index,
            candidate.third_index,
        ] {
            let branch = &branches[index];
            let role = branch_role(&candidate.label, &branch.branch);
            transforms.insert(
                index,
                create_branch_transform(
                    BranchTransformCategory::ThreeCombination,
                    role,
                    &candidate.element,
                    &format!("三合化{}", candidate.element),
                    &format!("三合化{}@three", candidate.element),
                    resolve_three_combination_factor(&candidate.label, role),
                    resolve_three_combination_residual_factor(&candidate.label, role),
                    branch,
                ),
            );
        }
    }
    transforms
}

fn resolve_half_combination_transforms(
    candidates: &[HalfCombinationCandidate],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    for candidate in candidates.iter().filter(|item| item.transforms) {
        if is_overpowered_by_tight_six_transformation(candidate, branches, stems, day_master)
            || is_suppressed_by_stacked_six_combination(candidate, branches)
            || three_combination_candidates.iter().any(|item| {
                item.involves(candidate.left_index) && item.involves(candidate.right_index)
            })
            // 三會即使不化，勢力仍大於半合；任一支落在三會之中，半合就不成立
            // （八字真理下冊 p222 鄧太：申酉戌三會不化，午戌不化火）。
            || three_meeting_candidates.iter().any(|item| {
                item.involves(candidate.left_index) || item.involves(candidate.right_index)
            })
        {
            continue;
        }
        for index in [candidate.left_index, candidate.right_index] {
            let branch = &branches[index];
            let role = if is_half_combination_specialist(&branch.branch, &candidate.element) {
                CombinationBranchRole::Specialist
            } else {
                CombinationBranchRole::Growth
            };
            merge_transform(
                &mut transforms,
                index,
                create_branch_transform(
                    BranchTransformCategory::HalfCombination,
                    role,
                    &candidate.element,
                    &candidate.note,
                    &(candidate.combine_note.clone() + "@half"),
                    resolve_half_combination_factor(candidate, branch),
                    resolve_half_combination_residual_factor(candidate, branch),
                    branch,
                ),
                branch,
            );
        }
    }
    transforms
}

/// 夾合：中間一支左右緊貼兩個相同地支，且與兩側都成六合（如未午未、寅亥寅）。
/// 回傳兩側地支的 position_index。只看原局四柱。
fn sandwiched_six_combination_sides(
    center: &BranchInput,
    branches: &[BranchInput],
) -> Option<(usize, usize)> {
    if is_floating_pillar(&center.pillar) || center.position_index == 0 {
        return None;
    }
    let find = |index: usize| {
        branches
            .iter()
            .find(|item| item.position_index == index && !is_floating_pillar(&item.pillar))
    };
    let left = find(center.position_index - 1)?;
    let right = find(center.position_index + 1)?;
    (left.branch == right.branch && is_six_combination(&center.branch, &left.branch))
        .then_some((left.position_index, right.position_index))
}

/// 夾合中選哪一側合化：取離化神天干較近的一側（八字真理上冊 p52 柯震東：
/// 化神在日元及時干，故以月日午未六合化土論，年未沒有變化）。
/// 兩側一樣近時兩側都化（上冊 p49 案例四：化神在日元，辰酉辰兩邊都化金）。
fn preferred_sandwich_side(
    left: usize,
    right: usize,
    element: &str,
    stems: &[StemInput],
    day_master: &str,
) -> Option<usize> {
    let mut positions: Vec<usize> = stems
        .iter()
        .filter(|stem| stem_element(&stem.stem) == element && !is_floating_pillar(&stem.pillar))
        .map(|stem| stem.position_index)
        .collect();
    if stem_element(day_master) == element {
        positions.push(2);
    }
    let nearest = |index: usize| {
        positions
            .iter()
            .map(|position| position.abs_diff(index))
            .min()
            .unwrap_or(usize::MAX)
    };
    match nearest(left).cmp(&nearest(right)) {
        std::cmp::Ordering::Less => Some(left),
        std::cmp::Ordering::Greater => Some(right),
        std::cmp::Ordering::Equal => None,
    }
}

/// 夾合中心另外還捲入其他合局（半合、三合、三會）時，多個合局互相牽制，均合而不化
/// （八字真理下冊 p257 例二：二寅夾合亥，又有亥卯半合）。
fn sandwich_center_has_competing_combination(
    center: &BranchInput,
    outer: (usize, usize),
    half_combination_candidates: &[HalfCombinationCandidate],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
) -> bool {
    let index = center.position_index;
    half_combination_candidates.iter().any(|candidate| {
        candidate.involves(index) && !candidate.involves(outer.0) && !candidate.involves(outer.1)
    }) || three_combination_candidates.iter().any(|item| item.involves(index))
        || three_meeting_candidates.iter().any(|item| item.involves(index))
}

fn resolve_six_combination_transforms(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
    half_combination_candidates: &[HalfCombinationCandidate],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    existing_transforms: &HashMap<usize, BranchTransform>,
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            if branch_pair_distance(left, right) != 1
                || !is_six_combination(&left.branch, &right.branch)
            {
                continue;
            }
            let element = resolve_six_combination_element(&left.branch, &right.branch);
            if existing_transforms
                .get(&left.position_index)
                .is_some_and(|item| {
                    !can_participate_in_second_transform(&left.branch, item, &element)
                })
                || existing_transforms
                    .get(&right.position_index)
                    .is_some_and(|item| {
                        !can_participate_in_second_transform(&right.branch, item, &element)
                    })
                || has_stronger_three_combination(left, right, three_combination_candidates)
                || has_stronger_three_meeting(left, right, three_meeting_candidates)
                || has_active_stronger_half_combination(
                    left,
                    right,
                    branches,
                    half_combination_candidates,
                )
                || !should_transform_branch_combination(
                    &element,
                    branches,
                    &[left.position_index, right.position_index],
                    stems,
                    day_master,
                )
            {
                continue;
            }
            // 夾合：只化靠近化神的一側；中心另有其他合局牽制則都不化。
            let sandwich = [left, right].into_iter().find_map(|center| {
                sandwiched_six_combination_sides(center, branches).map(|sides| (center, sides))
            });
            if let Some((center, sides)) = sandwich {
                if sandwich_center_has_competing_combination(
                    center,
                    sides,
                    half_combination_candidates,
                    three_combination_candidates,
                    three_meeting_candidates,
                ) {
                    continue;
                }
                let chosen = preferred_sandwich_side(sides.0, sides.1, &element, stems, day_master);
                let outer = if center.position_index == left.position_index {
                    right.position_index
                } else {
                    left.position_index
                };
                if chosen.is_some_and(|chosen| chosen != outer) {
                    continue;
                }
            }
            let pair = format_branch_pair(left.position_index, right.position_index, branches);
            for branch in [left, right] {
                let role = if is_six_combination_specialist(&branch.branch, &element) {
                    CombinationBranchRole::Specialist
                } else {
                    CombinationBranchRole::Storage
                };
                merge_transform(
                    &mut transforms,
                    branch.position_index,
                    create_branch_transform(
                        BranchTransformCategory::SixCombination,
                        role,
                        &element,
                        &format!("六合化{}", element),
                        &format!("{}六合化{}", pair, element),
                        resolve_six_combination_factor(branch, &element),
                        resolve_six_combination_residual_factor(branch, &element),
                        branch,
                    ),
                    branch,
                );
            }
        }
    }
    transforms
}

fn resolve_six_combination_effects(
    branches: &[BranchInput],
    half_combination_candidates: &[HalfCombinationCandidate],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> HashMap<usize, SixCombinationEffect> {
    let mut effects = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            if branch_pair_distance(left, right) != 1
                || branch_transforms.contains_key(&left.position_index)
                || branch_transforms.contains_key(&right.position_index)
                || !is_six_combination(&left.branch, &right.branch)
                || has_stronger_three_combination(left, right, three_combination_candidates)
                || has_stronger_three_meeting(left, right, three_meeting_candidates)
                || has_stronger_half_combination(left, right, branches, half_combination_candidates)
            {
                continue;
            }
            let pair = format_branch_pair(left.position_index, right.position_index, branches);
            let element = resolve_six_combination_element(&left.branch, &right.branch);
            // 夾合而不化：兩個六合各耗中支 0.48，中支只剩 0.04（下冊 p168-169 假從女命）。
            let remainder_for = |branch: &BranchInput| {
                if sandwiched_six_combination_sides(branch, branches).is_some() {
                    0.04
                } else {
                    0.52
                }
            };
            effects.insert(
                left.position_index,
                SixCombinationEffect {
                    other_index: right.position_index,
                    pair: pair.clone(),
                    element: element.clone(),
                    remainder: remainder_for(left),
                },
            );
            effects.insert(
                right.position_index,
                SixCombinationEffect {
                    other_index: left.position_index,
                    pair,
                    element,
                    remainder: remainder_for(right),
                },
            );
        }
    }
    effects
}

fn resolve_branch_prune_effects(
    branches: &[BranchInput],
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> HashMap<usize, BranchPruneEffect> {
    let mut effects = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            // 合化局力量勝過墓庫沖，化局成立時沖不需理會（上冊 p138）。
            if branch_transforms.contains_key(&left.position_index)
                || branch_transforms.contains_key(&right.position_index)
            {
                continue;
            }
            if is_chen_xu_pair(&left.branch, &right.branch)
                && branch_pair_distance(left, right) == 1
            {
                effects.insert(
                    left.position_index,
                    BranchPruneEffect {
                        remaining_stem: primary_hidden_stem(left),
                        remaining_ratio: primary_hidden_stem_ratio(left),
                        note: "辰戌近沖：旁氣化零，只保留主氣，並沿用主氣原比例".to_string(),
                    },
                );
                effects.insert(
                    right.position_index,
                    BranchPruneEffect {
                        remaining_stem: primary_hidden_stem(right),
                        remaining_ratio: primary_hidden_stem_ratio(right),
                        note: "辰戌近沖：旁氣化零，只保留主氣，並沿用主氣原比例".to_string(),
                    },
                );
            }
        }
    }
    for i in 0..branches.len().saturating_sub(2) {
        let left = &branches[i];
        let middle = &branches[i + 1];
        let right = &branches[i + 2];
        if [left, middle, right]
            .iter()
            .any(|item| is_floating_pillar(&item.pillar))
        {
            continue;
        }
        if left.branch == "寅" && middle.branch == "申" && right.branch == "寅" {
            effects.insert(
                middle.position_index,
                BranchPruneEffect {
                    remaining_stem: "戊".to_string(),
                    remaining_ratio: 0.25,
                    note: "二寅夾沖申：申支只留戊土旁氣".to_string(),
                },
            );
        }
    }
    effects
}

/// 六合不化逢專氣緊貼沖（上冊 p139）：以沖論，但合力 0.48 把受沖支拉回，
/// 二沖支各 ×0.48，六合另一支不變。回傳被緩和的沖（以位置配對表示）。
/// 三合不化：專氣支提升至 1.1，其他二支相關支氣由 0.25 提升至 0.4（上冊 p133）。
/// 三會不化：專氣支與長生支主氣提升至 1.45，墓庫支相關支氣由 0.25 提升至 0.5（上冊 p134）。
fn resolve_non_transform_boosts(
    branches: &[BranchInput],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> HashMap<usize, Vec<NonTransformBoost>> {
    let mut boosts: HashMap<usize, Vec<NonTransformBoost>> = HashMap::new();
    // 四庫匯聚而不化：各支土氣由 0.5 提升至 0.7（上冊 p55）。
    if let Some(indexes) = four_storage_indexes(branches) {
        if indexes.iter().all(|index| !branch_transforms.contains_key(index)) {
            for index in indexes {
                let branch = &branches[index];
                let share = element_share_in_branch(&branch.branch, "土");
                boosts.entry(index).or_default().push(NonTransformBoost {
                    element: "土".to_string(),
                    extra_units: 0.7 - share,
                    note: format!(
                        "辰戌丑未四庫匯聚不化，{}之土氣由 {} 提升至 0.7",
                        branch.branch,
                        round3_string(share)
                    ),
                });
            }
        }
    }
    // 兩個三合同時不化而互相重疊（量化五行力下冊 p183-184 阿儀：申子辰子）：
    // 局中水氣按合化局的六成推算，二子各提升至 1.3、申辰各提升至 0.5，每支只提升一次。
    let open_combinations: Vec<&ThreeCombinationCandidate> = three_combination_candidates
        .iter()
        .filter(|item| !item.transforms)
        .filter(|candidate| {
            !three_meeting_candidates.iter().any(|meeting| {
                meeting.involves(candidate.first_index)
                    || meeting.involves(candidate.second_index)
                    || meeting.involves(candidate.third_index)
            })
        })
        .collect();
    let doubled_combination = |candidate: &ThreeCombinationCandidate| {
        let own = [candidate.first_index, candidate.second_index, candidate.third_index];
        open_combinations.iter().any(|other| {
            !std::ptr::eq(*other, candidate)
                && other.element == candidate.element
                && [other.first_index, other.second_index, other.third_index]
                    .iter()
                    .any(|index| own.contains(index))
        })
    };
    let mut add_group = |indexes: [usize; 3],
                         element: &str,
                         label: &str,
                         meeting: bool,
                         doubled: bool| {
        if indexes
            .iter()
            .any(|index| branch_transforms.contains_key(index))
        {
            return;
        }
        for index in indexes {
            let branch = &branches[index];
            let share = element_share_in_branch(&branch.branch, element);
            if share <= 0.0 {
                continue;
            }
            let role = branch_role(label, &branch.branch);
            let target = match (meeting, role) {
                (false, CombinationBranchRole::Specialist) if doubled => 1.3,
                (false, _) if doubled => 0.5,
                (false, CombinationBranchRole::Specialist) => 1.1,
                (false, _) => 0.4,
                (true, CombinationBranchRole::Specialist | CombinationBranchRole::Growth) => 1.45,
                (true, _) => 0.5,
            };
            let extra_units = target - share;
            if extra_units <= 0.0 {
                continue;
            }
            // 同一支被兩個三會（如兩未夾巳午）或兩個三合重複牽涉時只提升一次（八字真理上冊 p52）。
            if (meeting || doubled)
                && boosts
                    .get(&index)
                    .is_some_and(|items| items.iter().any(|item| item.element == element))
            {
                continue;
            }
            boosts.entry(index).or_default().push(NonTransformBoost {
                element: element.to_string(),
                extra_units,
                note: format!(
                    "{}{}不化，{}之{}氣由 {} 提升至 {}",
                    label,
                    if meeting { "三會" } else { "三合" },
                    branch.branch,
                    element,
                    round3_string(share),
                    round3_string(target)
                ),
            });
        }
    };
    for candidate in three_meeting_candidates.iter().filter(|item| !item.transforms) {
        add_group(
            [candidate.first_index, candidate.second_index, candidate.third_index],
            &candidate.element,
            &candidate.label,
            true,
            false,
        );
    }
    for candidate in &open_combinations {
        add_group(
            [candidate.first_index, candidate.second_index, candidate.third_index],
            &candidate.element,
            &candidate.label,
            false,
            doubled_combination(candidate),
        );
    }
    boosts
}

fn element_share_in_branch(branch: &str, element: &str) -> f64 {
    let hidden_stems = hidden_stems_of(branch);
    let ratios = hidden_ratios(hidden_stems.len());
    hidden_stems
        .iter()
        .zip(ratios)
        .filter(|(stem, _)| stem_element(stem) == element)
        .map(|(_, ratio)| ratio)
        .sum()
}

fn build_boost_branch_qi_slices(
    input: &BranchInput,
    boosts: Option<&Vec<NonTransformBoost>>,
    day_master: &str,
    weight: f64,
) -> Vec<BranchQiSlice> {
    let Some(boosts) = boosts else {
        return Vec::new();
    };
    let hidden_stems = hidden_stems_of(&input.branch);
    boosts
        .iter()
        .filter_map(|boost| {
            let (index, stem) = hidden_stems
                .iter()
                .enumerate()
                .find(|(_, stem)| stem_element(stem) == boost.element)?;
            let ten_god = resolve_ten_god(day_master, stem);
            let raw_score = resolve_ten_god_score(&ten_god) * boost.extra_units;
            let position_adjusted = round1(raw_score / 100.0 * weight);
            Some(BranchQiSlice {
                display_stem: stem.clone(),
                ten_god,
                ratio: boost.extra_units,
                row_raw_score: raw_score,
                adjusted_raw_contribution: raw_score,
                position_adjusted,
                base_interaction: "不化提升".to_string(),
                base_note: format!("{} → 位調後 {:+.1}", boost.note, position_adjusted),
                transformed: false,
                hidden_stem_index: index as isize,
                transparency_key: stem.clone(),
                transparency_base: position_adjusted,
                transform_slice_kind: BranchTransformSliceKind::Residual,
            })
        })
        .collect()
}

fn is_pinched_by_specialist_clash(
    center: &BranchInput,
    branches: &[BranchInput],
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> bool {
    branches
        .iter()
        .filter(|other| {
            other.position_index != center.position_index
                && !is_floating_pillar(&other.pillar)
                && !is_floating_pillar(&center.pillar)
                && !branch_transforms.contains_key(&other.position_index)
                && branch_pair_distance(center, other) == 1
                && is_direct_clash(&center.branch, &other.branch)
                && classify_branch_clash(&center.branch, &other.branch)
                    == BranchClashType::Specialist
        })
        .count()
        >= 2
}

#[derive(Debug, Clone, Default)]
struct SixCombinationClashResolution {
    /// 專氣沖遇六合不化：以沖論，但合力緩沖，受沖二支 ×0.48（下冊 p139）。
    cushioned: HashSet<(usize, usize)>,
    /// 長生、墓庫沖遇六合不化：沖勝合，但六合力 0.48 抵消部分沖力（下冊 p190 B女士）。
    /// 值為受合之中心支。
    buffered: HashMap<(usize, usize), usize>,
    /// 失效的六合（被抵消，或兩支各自受沖）。
    voided_pairs: HashSet<(usize, usize)>,
}

fn ordered_pair(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

fn resolve_six_combination_clash_cushions(
    branches: &[BranchInput],
    six_combination_effects: &mut HashMap<usize, SixCombinationEffect>,
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> SixCombinationClashResolution {
    let mut resolution = SixCombinationClashResolution::default();
    let mut pairs: Vec<(usize, usize)> = six_combination_effects
        .iter()
        .map(|(index, effect)| ordered_pair(*index, effect.other_index))
        .collect();
    pairs.sort();
    pairs.dedup();
    let clashes_on = |index: usize, partner: usize| -> Vec<(usize, BranchClashType)> {
        let center = &branches[index];
        branches
            .iter()
            .filter(|other| {
                other.position_index != index
                    && other.position_index != partner
                    && !branch_transforms.contains_key(&other.position_index)
                    && branch_pair_distance(center, other) == 1
                    && is_direct_clash(&center.branch, &other.branch)
            })
            .map(|other| {
                (
                    other.position_index,
                    classify_branch_clash(&center.branch, &other.branch),
                )
            })
            .collect()
    };
    for (left, right) in pairs {
        let left_clashes = clashes_on(left, right);
        let right_clashes = clashes_on(right, left);
        if left_clashes.is_empty() && right_clashes.is_empty() {
            continue;
        }
        resolution.voided_pairs.insert((left, right));
        if !left_clashes.is_empty() && !right_clashes.is_empty() {
            // 六合二支各自受沖：沖力壓倒合力，以沖論、不計合（八字真理上冊 p30 沖剋男命）。
            continue;
        }
        let (center, clashes) = if left_clashes.is_empty() {
            (right, right_clashes)
        } else {
            (left, left_clashes)
        };
        for (other, clash_type) in clashes {
            let key = ordered_pair(center, other);
            if clash_type == BranchClashType::Specialist {
                resolution.cushioned.insert(key);
            } else {
                resolution.buffered.insert(key, center);
            }
        }
    }
    for (left, right) in &resolution.voided_pairs {
        six_combination_effects.remove(left);
        six_combination_effects.remove(right);
    }
    resolution
}

fn resolve_branch_qi_modifiers(
    branches: &[BranchInput],
    six_combination_effects: &HashMap<usize, SixCombinationEffect>,
    three_meeting_candidates: &[ThreeMeetingCandidate],
    branch_transforms: &HashMap<usize, BranchTransform>,
    six_clash_resolution: &SixCombinationClashResolution,
    day_master: &str,
) -> HashMap<usize, BranchQiModifier> {
    let mut modifiers = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            let dist = branch_pair_distance(left, right);
            if !is_direct_clash(&left.branch, &right.branch)
                || dist > 2
                || branch_transforms.contains_key(&left.position_index)
                || branch_transforms.contains_key(&right.position_index)
                || should_suppress_clash_by_six_combination(left, right, six_combination_effects)
                || should_suppress_clash_by_three_meeting(
                    left,
                    right,
                    dist,
                    three_meeting_candidates,
                )
            {
                continue;
            }
            let clash_type = classify_branch_clash(&left.branch, &right.branch);
            let base_multiplier = branch_clash_affected_multiplier(dist);
            let clash_key = ordered_pair(left.position_index, right.position_index);
            let buffered_multiplier = six_clash_resolution
                .buffered
                .get(&clash_key)
                .map(|center| {
                    let (center, other) = if *center == left.position_index {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    buffered_clash_multiplier(&center.branch, &other.branch)
                });
            let cushioned = six_clash_resolution.cushioned.contains(&clash_key);
            for (branch, other) in [(left, right), (right, left)] {
                // 二支同沖一支：被沖之支只按較近的一支計沖。
                // 一般情況較遠的沖者不受影響（上冊 p161 李國寶年卯、p71 阮玲玉時亥）；
                // 兩個沖者都是日元祿支時（下冊 p250 二巳沖一亥），按沖力分攤反作用：
                // 緊貼為 2 個專氣沖力、隔一位為 0.5 個，月巳承擔八成、年巳承擔二成。
                let target_far_dist =
                    second_clasher_distance(other, branch, branches, branch_transforms)
                        .filter(|other_dist| *other_dist < dist);
                let clasher_far_dist =
                    second_clasher_distance(branch, other, branches, branch_transforms)
                        .filter(|other_dist| *other_dist < dist);
                if target_far_dist.is_some() {
                    continue;
                }
                if clasher_far_dist.is_some() && !is_element_lu_branch(day_master, &branch.branch)
                {
                    continue;
                }
                let shared_multiplier = if is_element_lu_branch(day_master, &branch.branch) {
                    second_clasher_distance(branch, other, branches, branch_transforms).map(
                        |second_dist| {
                            let own = clash_force_by_distance(dist);
                            1.0 - own / (own + clash_force_by_distance(second_dist))
                        },
                    )
                } else {
                    None
                };
                let affected_multiplier = if let Some(multiplier) = shared_multiplier {
                    multiplier
                } else if let Some(multiplier) = buffered_multiplier {
                    multiplier
                } else if cushioned {
                    0.48
                } else if dist == 1
                    && clash_type == BranchClashType::Specialist
                    && is_pinched_by_specialist_clash(other, branches, branch_transforms)
                {
                    // 左沖右沖（夾沖，上冊 p145）：中間專氣支化零，兩側各只受一半之沖，×0.5。
                    0.5
                } else {
                    base_multiplier
                };
                let mut modifier = build_branch_clash_qi_modifier(
                    &branch.branch,
                    &other.branch,
                    clash_type,
                    affected_multiplier,
                    dist,
                );
                // 承擔大半沖力的祿支已無法壓制旁氣，藏干各自獨立推算，不再以 +100 計（下冊 p250 月巳）。
                if shared_multiplier.is_some_and(|multiplier| multiplier < 0.5) {
                    modifier.stem_multipliers.insert(LU_MAIN_QI_KEY.to_string(), 0.0);
                }
                merge_branch_qi_modifier_into(&mut modifiers, branch.position_index, modifier);
            }
        }
    }
    for i in 0..branches.len().saturating_sub(2) {
        let left = &branches[i];
        let middle = &branches[i + 1];
        let right = &branches[i + 2];
        if [left, middle, right]
            .iter()
            .any(|item| is_floating_pillar(&item.pillar))
        {
            continue;
        }
        if left.branch == "寅" && middle.branch == "申" && right.branch == "寅" {
            let mut element_multipliers = HashMap::new();
            element_multipliers.insert("木".to_string(), 0.5);
            element_multipliers.insert("火".to_string(), 0.5);
            let modifier = BranchQiModifier {
                stem_multipliers: HashMap::new(),
                element_multipliers,
                note: "二寅夾沖申：寅中木火之氣減半".to_string(),
            };
            merge_branch_qi_modifier_into(&mut modifiers, left.position_index, modifier.clone());
            merge_branch_qi_modifier_into(&mut modifiers, right.position_index, modifier);
        }
    }
    modifiers
}

fn resolve_stem_combination_effects(
    stems: &[StemInput],
    month_branch: &BranchInput,
    month_transform: Option<&BranchTransform>,
) -> HashMap<usize, StemCombinationEffect> {
    let mut effects = HashMap::new();
    for i in 0..stems.len() {
        for j in (i + 1)..stems.len() {
            let left = &stems[i];
            let right = &stems[j];
            if stem_pair_distance(left, right) != 1 {
                continue;
            }
            let Some((pair, target_element)) = resolve_stem_combination(&left.stem, &right.stem)
            else {
                continue;
            };
            let transforms =
                month_allows_stem_transformation(&target_element, month_branch, month_transform);
            let effect = StemCombinationEffect {
                pair: pair.clone(),
                target_element: target_element.clone(),
                remainder: if transforms { 1.0 } else { 0.52 },
                transforms,
                note: if transforms {
                    format!("{}合{}（工程近似：按化氣改論）", pair, target_element)
                } else {
                    format!("{}合{}（工程近似：合而不化）", pair, target_element)
                },
                combine_note: format!("{}@{}{}", pair, left.pillar, right.pillar),
            };
            effects.insert(left.position_index, effect.clone());
            effects.insert(right.position_index, effect);
        }
    }
    effects
}

fn resolve_stem_clash_effects(
    stems: &[StemInput],
    combination_effects: &mut HashMap<usize, StemCombinationEffect>,
) -> HashMap<usize, StemClashEffect> {
    let mut effects = HashMap::new();
    for i in 0..stems.len() {
        for j in (i + 1)..stems.len() {
            let left = &stems[i];
            let right = &stems[j];
            if !is_stem_clash(&left.stem, &right.stem) {
                continue;
            }
            let dist = stem_pair_distance(left, right);
            if !is_supported_stem_clash_distance(dist) {
                continue;
            }
            if dist == 2 && soften_combination_by_distant_clash(left, right, combination_effects) {
                continue;
            }
            let remainder = stem_clash_remainder(dist);
            let pair = format!("{}{}", left.stem, right.stem);
            let note = match dist {
                1 => format!("{}緊貼相沖", pair),
                2 => format!("{}隔一位相沖", pair),
                _ => format!("{}隔二位相沖", pair),
            };
            merge_stem_clash_effect_into(&mut effects, left.position_index, remainder, &note);
            merge_stem_clash_effect_into(&mut effects, right.position_index, remainder, &note);
        }
    }
    effects
}

fn resolve_stem_root(
    stem: &str,
    branch: &BranchInput,
    branch_transform: Option<&BranchTransform>,
    branch_prune_effect: Option<&BranchPruneEffect>,
    branch_qi_modifier: Option<&BranchQiModifier>,
) -> RootApplication {
    let element = stem_element(stem);
    if let Some(transform) = branch_transform {
        let mut best = RootApplication::none();
        for slice in &transform.slices {
            if !slice.transformed || slice.display_stem != element {
                continue;
            }
            let reason = match slice.kind {
                BranchTransformSliceKind::Primary => "主化五行祿根",
                BranchTransformSliceKind::Secondary => "次化五行祿根",
                BranchTransformSliceKind::Residual => "化後五行祿根",
            };
            best = stronger_root(
                best,
                RootApplication {
                    level: RootLevel::Lu,
                    support_stem: slice.display_stem.clone(),
                    reason: reason.to_string(),
                    branch: branch.branch.clone(),
                    qi_multiplier: slice.root_factor,
                },
            );
        }
        let residual_factor = transform
            .slices
            .iter()
            .filter(|slice| !slice.transformed)
            .map(|slice| slice.factor)
            .fold(0.0, f64::max);
        if residual_factor > 0.0 {
            let residual_root =
                resolve_natural_stem_root(stem, branch, branch_prune_effect, branch_qi_modifier);
            if residual_root.level != RootLevel::None {
                best = stronger_root(
                    best,
                    RootApplication {
                        level: residual_root.level,
                        support_stem: residual_root.support_stem,
                        reason: format!("{}餘氣", residual_root.reason),
                        branch: residual_root.branch,
                        qi_multiplier: residual_root.qi_multiplier * residual_factor,
                    },
                );
            }
        }
        return best;
    }
    resolve_natural_stem_root(stem, branch, branch_prune_effect, branch_qi_modifier)
}

fn resolve_natural_stem_root(
    stem: &str,
    branch: &BranchInput,
    branch_prune_effect: Option<&BranchPruneEffect>,
    branch_qi_modifier: Option<&BranchQiModifier>,
) -> RootApplication {
    let element = stem_element(stem);
    let target_branch = branch.branch.as_str();
    let mut best = RootApplication::none();
    if lu_root_branches(&element).contains(&target_branch) {
        best = stronger_root(
            best,
            RootApplication {
                level: RootLevel::Lu,
                support_stem: element.clone(),
                reason: "祿根表".to_string(),
                branch: target_branch.to_string(),
                qi_multiplier: 1.0,
            },
        );
    }
    if middle_root_branches(&element).contains(&target_branch) {
        best = stronger_root(
            best,
            RootApplication {
                level: RootLevel::Middle,
                support_stem: element.clone(),
                reason: "中根表".to_string(),
                branch: target_branch.to_string(),
                qi_multiplier: 1.0,
            },
        );
    }
    if minor_root_branches(&element).contains(&target_branch) {
        best = stronger_root(
            best,
            RootApplication {
                level: RootLevel::Minor,
                support_stem: element.clone(),
                reason: "微根表".to_string(),
                branch: target_branch.to_string(),
                qi_multiplier: 1.0,
            },
        );
    }
    if tomb_root_branches(&element).contains(&target_branch) {
        best = stronger_root(
            best,
            RootApplication {
                level: RootLevel::Tomb,
                support_stem: element.clone(),
                reason: "墓根表".to_string(),
                branch: target_branch.to_string(),
                qi_multiplier: 1.0,
            },
        );
    }
    let hidden_stems = if let Some(effect) = branch_prune_effect {
        vec![effect.remaining_stem.clone()]
    } else if branch.hidden_stems.is_empty() {
        hidden_stems_of(&branch.branch)
    } else {
        branch.hidden_stems.clone()
    };
    for (index, hidden_stem) in hidden_stems.iter().enumerate() {
        if hidden_stem == stem {
            best = stronger_root(
                best,
                RootApplication {
                    level: if index == 0 {
                        RootLevel::Middle
                    } else {
                        RootLevel::Minor
                    },
                    support_stem: hidden_stem.clone(),
                    reason: if index == 0 {
                        "藏干同干（主氣）".to_string()
                    } else {
                        "藏干同干".to_string()
                    },
                    branch: target_branch.to_string(),
                    qi_multiplier: 1.0,
                },
            );
            continue;
        }
        if stem_element(hidden_stem) == element {
            best = stronger_root(
                best,
                RootApplication {
                    level: RootLevel::Minor,
                    support_stem: hidden_stem.clone(),
                    reason: if index == 0 {
                        "藏干同五行（主氣）".to_string()
                    } else {
                        "藏干同五行".to_string()
                    },
                    branch: target_branch.to_string(),
                    qi_multiplier: 1.0,
                },
            );
        }
    }
    if best.level == RootLevel::None {
        return best;
    }
    let qi_multiplier = branch_qi_multiplier(branch_qi_modifier, stem);
    if qi_multiplier <= 0.0 {
        return RootApplication::none();
    }
    if (qi_multiplier - 1.0).abs() > 0.0001 {
        best.qi_multiplier = qi_multiplier;
        best.reason = format!("{}，支氣折減", best.reason);
    }
    best
}

fn resolve_branch_transparencies(
    hidden_stem: &str,
    hidden_stem_index: isize,
    branch: &BranchInput,
    stems: &[StemInput],
    stem_combination_effects: &HashMap<usize, StemCombinationEffect>,
    stem_clash_effects: &HashMap<usize, StemClashEffect>,
    branch_prune_effects: &HashMap<usize, BranchPruneEffect>,
    position_adjusted: f64,
) -> Vec<TransparencyApplication> {
    let hidden_element = stem_element(hidden_stem);
    let mut matches = Vec::new();
    for stem in stems {
        if branch_prune_blocks_stem(
            branch_prune_effects.get(&branch.position_index),
            hidden_stem,
        ) {
            break;
        }
        let visible_element = effective_stem_element(stem, stem_combination_effects);
        if visible_element != hidden_element {
            continue;
        }
        let dist = stem_branch_distance(stem, branch);
        let mut multiplier = transparency_multiplier(dist);
        let mut kind = if stem.stem == hidden_stem {
            "同干透出".to_string()
        } else if hidden_stem_index == 0 {
            "主氣同五行透干".to_string()
        } else {
            "同五行透干".to_string()
        };
        if let Some(effect) = stem_combination_effects.get(&stem.position_index) {
            multiplier *= effect.remainder;
            kind = format!("{}（{}合後餘力）", kind, effect.pair);
        }
        if let Some(effect) = stem_clash_effects.get(&stem.position_index) {
            multiplier *= effect.remainder;
            kind = format!("{}（{}）", kind, effect.note);
        }
        matches.push(TransparencyApplication {
            stem_pillar: stem.pillar.clone(),
            visible_stem: stem.stem.clone(),
            element: hidden_element.clone(),
            multiplier,
            bonus: round1(round1(position_adjusted) * multiplier),
            kind,
        });
    }
    matches
}

/// 天干合化成功後按化後五行論得透（上冊 p36）。
fn effective_stem_element(
    stem: &StemInput,
    stem_combination_effects: &HashMap<usize, StemCombinationEffect>,
) -> String {
    match stem_combination_effects.get(&stem.position_index) {
        Some(effect) if effect.transforms => effect.target_element.clone(),
        _ => stem_element(&stem.stem),
    }
}

fn resolve_transformed_branch_transparencies(
    transformed_element: &str,
    slice_kind: BranchTransformSliceKind,
    branch: &BranchInput,
    stems: &[StemInput],
    stem_combination_effects: &HashMap<usize, StemCombinationEffect>,
    stem_clash_effects: &HashMap<usize, StemClashEffect>,
    position_adjusted: f64,
) -> Vec<TransparencyApplication> {
    let mut matches = Vec::new();
    for stem in stems {
        if effective_stem_element(stem, stem_combination_effects) != transformed_element {
            continue;
        }
        let dist = stem_branch_distance(stem, branch);
        let mut multiplier = transparency_multiplier(dist);
        let mut kind = match slice_kind {
            BranchTransformSliceKind::Primary => "主化同五行透干".to_string(),
            BranchTransformSliceKind::Secondary => "次化同五行透干".to_string(),
            BranchTransformSliceKind::Residual => "化後同五行透干".to_string(),
        };
        if let Some(effect) = stem_combination_effects.get(&stem.position_index) {
            multiplier *= effect.remainder;
            kind = format!("{}（{}合後餘力）", kind, effect.pair);
        }
        if let Some(effect) = stem_clash_effects.get(&stem.position_index) {
            multiplier *= effect.remainder;
            kind = format!("{}（{}）", kind, effect.note);
        }
        matches.push(TransparencyApplication {
            stem_pillar: stem.pillar.clone(),
            visible_stem: stem.stem.clone(),
            element: transformed_element.to_string(),
            multiplier,
            bonus: round1(round1(position_adjusted) * multiplier),
            kind,
        });
    }
    matches
}

fn resolve_branch_interaction(
    input: &BranchInput,
    state: &ResolvedChartState,
) -> BranchInteraction {
    if state.branch_transforms.contains_key(&input.position_index) {
        return BranchInteraction::none();
    }
    if is_protected_by_non_transform_three_meeting(input, &state.three_meeting_candidates) {
        return BranchInteraction::none();
    }
    if let Some(effect) = state.six_combination_effects.get(&input.position_index) {
        let other = &state.branches[effect.other_index];
        return BranchInteraction::combine("近合".to_string(), effect.remainder, other);
    }
    let mut best = BranchInteraction::none();
    for other in &state.branches {
        if other.position_index == input.position_index
            || state.branch_transforms.contains_key(&other.position_index)
        {
            continue;
        }
        let dist = branch_pair_distance(input, other);
        if is_direct_clash(&input.branch, &other.branch) {
            if is_chen_xu_pair(&input.branch, &other.branch) && dist >= 2 {
                continue;
            }
            if should_suppress_clash_by_six_combination(
                input,
                other,
                &state.six_combination_effects,
            ) || should_suppress_clash_by_three_meeting(
                input,
                other,
                dist,
                &state.three_meeting_candidates,
            ) || state
                .branch_prune_effects
                .contains_key(&input.position_index)
                || state
                    .branch_prune_effects
                    .contains_key(&other.position_index)
            {
                continue;
            }
            let clash_type = classify_branch_clash(&input.branch, &other.branch);
            let multiplier = branch_clash_score_multiplier(clash_type, dist);
            let note = branch_clash_note(clash_type, dist);
            let candidate = BranchInteraction::clash(note, multiplier, other);
            if dist <= 1 {
                return candidate;
            }
            best = stronger_interaction(best, candidate);
            continue;
        }
        if is_six_combination(&input.branch, &other.branch) {
            let in_cushioned_clash = state
                .six_clash_resolution
                .cushioned
                .iter()
                .any(|(left, right)| [*left, *right].iter().any(|index| {
                    *index == input.position_index || *index == other.position_index
                }))
                || state
                    .six_clash_resolution
                    .voided_pairs
                    .contains(&ordered_pair(input.position_index, other.position_index));
            if in_cushioned_clash
                || has_stronger_three_combination(input, other, &state.three_combination_candidates)
                || has_stronger_three_meeting(input, other, &state.three_meeting_candidates)
                || has_stronger_half_combination(
                    input,
                    other,
                    &state.branches,
                    &state.half_combination_candidates,
                )
            {
                continue;
            }
            // 六合屬偶合，須緊貼才論合而不化（×0.52）；隔一位之合不需理會（上冊 p46、p128）。
            let candidate = if dist <= 1 {
                BranchInteraction::combine("近合".to_string(), 0.52, other)
            } else {
                continue;
            };
            best = stronger_interaction(best, candidate);
        }
    }
    best
}

fn stronger_interaction(
    current: BranchInteraction,
    candidate: BranchInteraction,
) -> BranchInteraction {
    if candidate.priority > current.priority {
        candidate
    } else {
        current
    }
}

fn stronger_root(current: RootApplication, candidate: RootApplication) -> RootApplication {
    if candidate.level.rank() > current.level.rank() {
        candidate
    } else {
        current
    }
}

fn should_transform_branch_combination(
    target_element: &str,
    branches: &[BranchInput],
    participants: &[usize],
    stems: &[StemInput],
    day_master: &str,
) -> bool {
    if !has_visible_transforming_stem(target_element, stems, day_master) {
        return false;
    }
    let Some(month_branch) = branches.iter().find(|item| item.pillar == "月支") else {
        return true;
    };
    let month_main_stem = resolve_month_main_stem(month_branch);
    let month_element = stem_element(&month_main_stem);
    if controls(&month_element, target_element)
        && month_can_block_transformation(month_branch, branches, participants)
    {
        return false;
    }
    !stems_block_transformation(target_element, stems)
}

/// 月令主氣能否擋住合化（八字真理下冊 p235 作者補充）：
/// （月令本身參與合局時仍以其主氣論，如上冊 p109 巳月巳酉半合不化）；
/// 專氣月令受鄰支沖則受損，不能阻化，合而不化仍可阻化；
/// 非專氣月令與鄰支相沖或相合，力量少於 0.5 個專氣，不能阻化（下冊 p238）。
fn month_can_block_transformation(
    month_branch: &BranchInput,
    branches: &[BranchInput],
    participants: &[usize],
) -> bool {
    let specialist = matches!(month_branch.branch.as_str(), "子" | "午" | "卯" | "酉");
    let neighbours = branches.iter().filter(|other| {
        other.position_index != month_branch.position_index
            && !is_floating_pillar(&other.pillar)
            && !participants.contains(&other.position_index)
            && branch_pair_distance(month_branch, other) == 1
    });
    for other in neighbours {
        if is_direct_clash(&month_branch.branch, &other.branch) {
            return false;
        }
        if !specialist && is_six_combination(&month_branch.branch, &other.branch) {
            return false;
        }
    }
    true
}

/// 年月二干或月時二干同剋化神，份量相加達月令的 100，可擋住地支合化
/// （八字真理上冊 p94-95、下冊 p222 阿宏：丙丁緊貼，辰酉、酉丑均不可化金）。
fn stems_block_transformation(target_element: &str, stems: &[StemInput]) -> bool {
    let weight: f64 = stems
        .iter()
        .filter(|stem| !is_floating_pillar(&stem.pillar))
        .filter(|stem| controls(&stem_element(&stem.stem), target_element))
        .map(|stem| position_weight(&stem.pillar))
        .sum();
    weight >= 100.0
}

fn has_visible_transforming_stem(
    target_element: &str,
    stems: &[StemInput],
    day_master: &str,
) -> bool {
    stem_element(day_master) == target_element
        || stems
            .iter()
            .any(|item| stem_element(&item.stem) == target_element)
}

fn create_branch_transform(
    category: BranchTransformCategory,
    role: CombinationBranchRole,
    element: &str,
    note: &str,
    combine_note: &str,
    factor: f64,
    residual_factor: f64,
    branch: &BranchInput,
) -> BranchTransform {
    let mut slices = vec![BranchTransformSlice {
        kind: if matches!(role, CombinationBranchRole::Specialist) {
            BranchTransformSliceKind::Primary
        } else {
            BranchTransformSliceKind::Secondary
        },
        display_stem: element.to_string(),
        ratio: 1.0,
        transformed: true,
        hidden_stem_index: -1,
        interaction: note.to_string(),
        factor,
        transparency_base_factor: factor,
        root_factor: if category == BranchTransformCategory::SixCombination
            && residual_factor <= 0.0
        {
            1.0
        } else {
            factor
        },
        note: note.to_string(),
    }];
    if residual_factor > 0.0 {
        let hidden_stems = if branch.hidden_stems.is_empty() {
            hidden_stems_of(&branch.branch)
        } else {
            branch.hidden_stems.clone()
        };
        let ratios = hidden_ratios(hidden_stems.len());
        for (index, hidden_stem) in hidden_stems.iter().enumerate() {
            slices.push(BranchTransformSlice {
                kind: BranchTransformSliceKind::Residual,
                display_stem: hidden_stem.clone(),
                ratio: ratios[index],
                transformed: false,
                hidden_stem_index: index as isize,
                interaction: format!("{}餘氣", branch.branch),
                factor: residual_factor,
                transparency_base_factor: 1.0,
                root_factor: residual_factor,
                note: format!("{}餘氣", branch.branch),
            });
        }
    }
    BranchTransform {
        category,
        role,
        element: element.to_string(),
        note: note.to_string(),
        combine_note: combine_note.to_string(),
        factor,
        residual_factor,
        slices,
    }
}

fn merge_missing_transforms(
    target: &mut HashMap<usize, BranchTransform>,
    additions: HashMap<usize, BranchTransform>,
) {
    for (index, transform) in additions {
        target.entry(index).or_insert(transform);
    }
}

fn merge_transforms(
    target: &mut HashMap<usize, BranchTransform>,
    additions: HashMap<usize, BranchTransform>,
    branches: &[BranchInput],
) {
    for (index, transform) in additions {
        merge_transform(target, index, transform, &branches[index]);
    }
}

fn merge_transform(
    target: &mut HashMap<usize, BranchTransform>,
    index: usize,
    addition: BranchTransform,
    branch: &BranchInput,
) {
    let Some(current) = target.get(&index).cloned() else {
        target.insert(index, addition);
        return;
    };
    if current.element != addition.element {
        return;
    }
    if !can_participate_in_second_transform(&branch.branch, &current, &addition.element) {
        return;
    }
    if current.category == BranchTransformCategory::HalfCombination
        && addition.category == BranchTransformCategory::HalfCombination
        && current.role == CombinationBranchRole::Specialist
    {
        let merged_factor = current.factor + addition.factor - 1.0;
        target.insert(
            index,
            create_branch_transform(
                current.category.clone(),
                current.role,
                &current.element,
                &(current.note.clone() + "疊加"),
                &merge_combine_note(&current.combine_note, &addition.combine_note),
                merged_factor,
                current.residual_factor,
                branch,
            ),
        );
        return;
    }
    if current.role == CombinationBranchRole::Specialist
        && addition.category == BranchTransformCategory::SixCombination
    {
        target.insert(
            index,
            create_branch_transform(
                current.category.clone(),
                current.role,
                &current.element,
                &(current.note.clone() + "；可再接六合"),
                &merge_combine_note(&current.combine_note, &addition.combine_note),
                current.factor,
                current.residual_factor,
                branch,
            ),
        );
    }
}

fn merge_combine_note(left: &str, right: &str) -> String {
    if left == right || right.is_empty() {
        left.to_string()
    } else if left.is_empty() {
        right.to_string()
    } else {
        format!("{}；{}", left, right)
    }
}

fn can_participate_in_second_transform(
    original_branch: &str,
    existing: &BranchTransform,
    target_element: &str,
) -> bool {
    existing.element == target_element
        && resolve_branch_element(original_branch) == target_element
        && is_specialist_branch(original_branch)
}

/// 半合隔二位化只有 1.625 個五行專氣，不敵緊貼六合化的 1.65-1.8（上冊 p130）。
fn is_overpowered_by_tight_six_transformation(
    candidate: &HalfCombinationCandidate,
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
) -> bool {
    if normalize_distance(candidate.distance) < 3 {
        return false;
    }
    [candidate.left_index, candidate.right_index]
        .into_iter()
        .any(|index| {
            let branch = &branches[index];
            branches.iter().any(|other| {
                other.position_index != branch.position_index
                    && !candidate.involves(other.position_index)
                    && branch_pair_distance(branch, other) == 1
                    && is_six_combination(&branch.branch, &other.branch)
                    && should_transform_branch_combination(
                        &resolve_six_combination_element(&branch.branch, &other.branch),
                        branches,
                        &[branch.position_index, other.position_index],
                        stems,
                        day_master,
                    )
            })
        })
}

fn is_suppressed_by_stacked_six_combination(
    candidate: &HalfCombinationCandidate,
    branches: &[BranchInput],
) -> bool {
    let left = &branches[candidate.left_index];
    let right = &branches[candidate.right_index];
    stacked_six_combination_power_for_pair(left, right, branches)
        > half_combination_power(candidate)
}

fn has_active_stronger_half_combination(
    left: &BranchInput,
    right: &BranchInput,
    branches: &[BranchInput],
    half_combination_candidates: &[HalfCombinationCandidate],
) -> bool {
    half_combination_candidates.iter().any(|candidate| {
        (candidate.involves(left.position_index) || candidate.involves(right.position_index))
            && !(candidate.involves(left.position_index)
                && candidate.involves(right.position_index))
            && stacked_six_combination_power_for_pair(left, right, branches)
                < half_combination_power(candidate)
            && normalize_distance(candidate.distance) < 3
            && (candidate.distance <= 1 || candidate.transforms)
    })
}

fn has_stronger_half_combination(
    input: &BranchInput,
    other: &BranchInput,
    branches: &[BranchInput],
    candidates: &[HalfCombinationCandidate],
) -> bool {
    candidates.iter().any(|candidate| {
        (candidate.involves(input.position_index) || candidate.involves(other.position_index))
            && !(candidate.involves(input.position_index)
                && candidate.involves(other.position_index))
            && stacked_six_combination_power_for_pair(input, other, branches)
                < half_combination_power(candidate)
            && (candidate.distance <= 1 || candidate.transforms)
    })
}

fn has_stronger_three_combination(
    input: &BranchInput,
    other: &BranchInput,
    candidates: &[ThreeCombinationCandidate],
) -> bool {
    candidates.iter().any(|candidate| {
        (candidate.involves(input.position_index) || candidate.involves(other.position_index))
            && !(candidate.involves(input.position_index)
                && candidate.involves(other.position_index))
    })
}

/// 三會不化仍有 3.41 個五行專氣，勝過六合化（上冊 p134），
/// 包括六合二支本身就在三會之中（如巳午未中的午未）。
fn has_stronger_three_meeting(
    input: &BranchInput,
    other: &BranchInput,
    candidates: &[ThreeMeetingCandidate],
) -> bool {
    candidates.iter().any(|candidate| {
        candidate.involves(input.position_index) || candidate.involves(other.position_index)
    })
}

fn should_suppress_clash_by_six_combination(
    input: &BranchInput,
    other: &BranchInput,
    effects: &HashMap<usize, SixCombinationEffect>,
) -> bool {
    branch_pair_distance(input, other) > 1
        && (effects.contains_key(&input.position_index)
            || effects.contains_key(&other.position_index))
}

fn should_suppress_clash_by_three_meeting(
    input: &BranchInput,
    other: &BranchInput,
    dist: usize,
    candidates: &[ThreeMeetingCandidate],
) -> bool {
    dist > 1
        && candidates
            .iter()
            .filter(|candidate| !candidate.transforms)
            .any(|candidate| {
                candidate.involves(input.position_index) || candidate.involves(other.position_index)
            })
}

fn is_protected_by_non_transform_three_meeting(
    input: &BranchInput,
    candidates: &[ThreeMeetingCandidate],
) -> bool {
    candidates
        .iter()
        .any(|candidate| !candidate.transforms && candidate.involves(input.position_index))
}

/// 緊貼五合不化又遇隔一位相沖（上冊 p146）：合力 0.48 抵消沖力 0.125，
/// 合中二干只承受 0.355 個合力，各 ×0.645；隔位沖干的沖力已被消弭，不扣分。
fn soften_combination_by_distant_clash(
    left: &StemInput,
    right: &StemInput,
    combination_effects: &mut HashMap<usize, StemCombinationEffect>,
) -> bool {
    let combined = [left, right].into_iter().find(|stem| {
        combination_effects
            .get(&stem.position_index)
            .is_some_and(|effect| !effect.transforms)
    });
    let Some(combined) = combined else {
        return false;
    };
    let pair = combination_effects[&combined.position_index].pair.clone();
    for effect in combination_effects.values_mut() {
        if effect.pair == pair && !effect.transforms {
            effect.remainder = 0.645;
            effect.note = format!("{}合而不化，又受隔一位沖，合力抵消後各×0.645", pair);
        }
    }
    true
}

fn merge_stem_clash_effect_into(
    target: &mut HashMap<usize, StemClashEffect>,
    index: usize,
    remainder: f64,
    note: &str,
) {
    match target.get(&index) {
        Some(current) => {
            target.insert(
                index,
                StemClashEffect {
                    remainder: round3(current.remainder * remainder),
                    note: format!("{}；{}", current.note, note),
                },
            );
        }
        None => {
            target.insert(
                index,
                StemClashEffect {
                    remainder,
                    note: note.to_string(),
                },
            );
        }
    }
}

fn merge_branch_qi_modifier_into(
    target: &mut HashMap<usize, BranchQiModifier>,
    index: usize,
    incoming: BranchQiModifier,
) {
    let merged = if let Some(current) = target.get(&index) {
        merge_branch_qi_modifier(current, &incoming)
    } else {
        incoming
    };
    target.insert(index, merged);
}

fn merge_branch_qi_modifier(
    current: &BranchQiModifier,
    incoming: &BranchQiModifier,
) -> BranchQiModifier {
    let mut stem_multipliers = current.stem_multipliers.clone();
    for (key, value) in &incoming.stem_multipliers {
        let merged = stem_multipliers.get(key).copied().unwrap_or(1.0) * value;
        stem_multipliers.insert(key.clone(), merged);
    }
    let mut element_multipliers = current.element_multipliers.clone();
    for (key, value) in &incoming.element_multipliers {
        let merged = element_multipliers.get(key).copied().unwrap_or(1.0) * value;
        element_multipliers.insert(key.clone(), merged);
    }
    BranchQiModifier {
        stem_multipliers,
        element_multipliers,
        note: if current.note.is_empty() {
            incoming.note.clone()
        } else if incoming.note.is_empty() {
            current.note.clone()
        } else {
            format!("{}；{}", current.note, incoming.note)
        },
    }
}

/// 六合力緩沖長生、墓庫沖（下冊 p190 B女士）：中心支受沖的份量 f（巳亥沖時亥受沖 0.75），
/// 抵消 0.48 個合力後，受沖支氣只損 (f − 0.48) / f，兩支受沖之氣同乘此餘力。
/// 丑未沖遇子丑合時 f = 0.5，幾乎完全抵消（八字真理上冊 p136 胡適：以無沖無合推算）。
/// 與 clasher 同字、同沖 target、但距離不同的另一支：回傳該支與 target 的距離。
fn second_clasher_distance(
    clasher: &BranchInput,
    target: &BranchInput,
    branches: &[BranchInput],
    branch_transforms: &HashMap<usize, BranchTransform>,
) -> Option<usize> {
    let own_dist = branch_pair_distance(clasher, target);
    branches
        .iter()
        .filter(|item| {
            item.position_index != clasher.position_index
                && item.position_index != target.position_index
                && item.branch == clasher.branch
                && !is_floating_pillar(&item.pillar)
                && !branch_transforms.contains_key(&item.position_index)
        })
        .map(|item| branch_pair_distance(item, target))
        .find(|dist| (1..=2).contains(dist) && *dist != own_dist)
}

fn clash_force_by_distance(dist: usize) -> f64 {
    if dist <= 1 {
        2.0
    } else {
        0.5
    }
}

fn buffered_clash_multiplier(center: &str, other: &str) -> f64 {
    let affected: f64 = ["木", "火", "金", "水"]
        .iter()
        .map(|element| {
            element_share_in_branch(center, element)
                .min(element_share_in_branch(other, clash_opponent_element(element)))
        })
        .sum();
    if affected <= 0.48 {
        1.0
    } else {
        1.0 - (affected - 0.48) / affected
    }
}

const LU_MAIN_QI_KEY: &str = "祿主氣";

/// 長生、墓庫沖中互沖的五行：木↔金、火↔水。
fn clash_opponent_element(element: &str) -> &'static str {
    match element {
        "木" => "金",
        "金" => "木",
        "火" => "水",
        "水" => "火",
        _ => "",
    }
}

fn build_branch_clash_qi_modifier(
    branch: &str,
    other_branch: &str,
    clash_type: BranchClashType,
    affected_multiplier: f64,
    dist: usize,
) -> BranchQiModifier {
    // 量化五行力不分陰陽（上冊 p20、p28）：受沖支氣按五行折減，
    // 例如寅申沖時寅中木火之氣化零，乙在寅亦不能再得根。
    let affected_elements: Vec<String> = match clash_type {
        BranchClashType::Specialist => {
            let mut elements = vec![resolve_branch_element(branch)];
            for hidden in hidden_stems_of(branch) {
                elements.push(stem_element(&hidden));
            }
            elements
        }
        // 長生沖：主氣與一旁氣化零，戊土旁氣不受影響；墓庫沖：二旁氣化零，主氣土不受影響。
        BranchClashType::Growth | BranchClashType::Storage => match branch {
            "寅" => vec!["木", "火"],
            "申" => vec!["金", "水"],
            "巳" => vec!["火", "金"],
            "亥" => vec!["水", "木"],
            "辰" => vec!["木", "水"],
            "戌" => vec!["金", "火"],
            "丑" => vec!["水", "金"],
            "未" => vec!["火", "木"],
            _ => Vec::new(),
        }
        .into_iter()
        .map(str::to_string)
        .collect(),
    };
    let mut stem_multipliers = HashMap::new();
    // 祿支以主氣論：主氣受沖，祿神亦按沖的倍率折減（下冊 p171：亥為壬祿，主氣受沖，本分 ×0.75）。
    if affected_elements.contains(&resolve_branch_element(branch)) {
        stem_multipliers.insert(LU_MAIN_QI_KEY.to_string(), affected_multiplier);
    }
    let mut element_multipliers = HashMap::new();
    for element in affected_elements {
        // 長生沖拆成干沖（巳亥沖 = 丙壬沖 + 甲庚沖）：一氣只受對方相沖之氣的份量影響
        // （下冊 p171 N女士：亥中甲 0.37，只被巳中庚 0.25 所沖，甲剩 0.25×0.75 + 0.12 = 0.308）。
        let multiplier = if clash_type == BranchClashType::Growth {
            let own = element_share_in_branch(branch, &element);
            let opponent = element_share_in_branch(other_branch, clash_opponent_element(&element));
            if own > opponent && own > 0.0 {
                (opponent * affected_multiplier + (own - opponent)) / own
            } else {
                affected_multiplier
            }
        } else {
            affected_multiplier
        };
        element_multipliers.insert(element, multiplier);
    }
    let note = match clash_type {
        BranchClashType::Specialist => {
            clash_type_distance_note("專氣相沖", dist, affected_multiplier)
        }
        BranchClashType::Growth => clash_type_distance_note("長生相沖", dist, affected_multiplier),
        BranchClashType::Storage => clash_type_distance_note("墓庫相沖", dist, affected_multiplier),
    };
    BranchQiModifier {
        stem_multipliers,
        element_multipliers,
        note,
    }
}

fn resolve_print_presence(
    branch_scores: &[QuantModelPillarScore],
    stem_scores: &[QuantModelPillarScore],
) -> String {
    if stem_scores
        .iter()
        .any(|score| is_seal_god(&score.ten_god) && score.final_score > 0.0)
    {
        "天干見印".to_string()
    } else if branch_scores.iter().any(|pillar| {
        pillar.rows.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.ten_god
                    .as_ref()
                    .is_some_and(|ten_god| is_seal_god(ten_god))
                    && row.final_contribution.unwrap_or(0.0) > 0.0
            })
        })
    }) {
        "印只在地支".to_string()
    } else {
        "局中無印".to_string()
    }
}

fn aggregate_family_scores(
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> HashMap<&'static str, f64> {
    let mut totals = HashMap::from([
        ("比劫", 0.0),
        ("印", 0.0),
        ("食傷", 0.0),
        ("財", 0.0),
        ("官殺", 0.0),
    ]);
    for score in stem_scores {
        if let Some(family) = family_of(&score.ten_god) {
            *totals.entry(family).or_insert(0.0) += score.final_score;
        }
    }
    for pillar in branch_scores {
        if let Some(rows) = &pillar.rows {
            for row in rows {
                if let Some(ten_god) = &row.ten_god {
                    if let Some(family) = family_of(ten_god) {
                        *totals.entry(family).or_insert(0.0) +=
                            row.final_contribution.unwrap_or(0.0);
                    }
                }
            }
        }
    }
    totals
}

fn aggregate_element_scores(
    day_master: &str,
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> HashMap<String, f64> {
    let mut totals = HashMap::from([
        ("木".to_string(), 0.0),
        ("火".to_string(), 0.0),
        ("土".to_string(), 0.0),
        ("金".to_string(), 0.0),
        ("水".to_string(), 0.0),
    ]);
    for score in stem_scores {
        let element = stem_element(&score.target);
        *totals.entry(element).or_insert(0.0) += score.final_score;
    }
    let day_element = stem_element(day_master);
    for pillar in branch_scores {
        if let Some(rows) = &pillar.rows {
            for row in rows {
                let Some(hidden_stem) = &row.hidden_stem else {
                    continue;
                };
                let element = if hidden_stem == "祿" {
                    day_element.clone()
                } else if ["木", "火", "土", "金", "水"].contains(&hidden_stem.as_str()) {
                    hidden_stem.clone()
                } else {
                    stem_element(hidden_stem)
                };
                *totals.entry(element).or_insert(0.0) += row.final_contribution.unwrap_or(0.0);
            }
        }
    }
    totals
}

fn aggregate_element_magnitude_scores(
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> HashMap<String, f64> {
    let mut totals = HashMap::from([
        ("木".to_string(), 0.0),
        ("火".to_string(), 0.0),
        ("土".to_string(), 0.0),
        ("金".to_string(), 0.0),
        ("水".to_string(), 0.0),
    ]);
    for score in stem_scores {
        let element = stem_element(&score.target);
        if !element.is_empty() {
            *totals.entry(element).or_insert(0.0) += score.final_score.abs();
        }
    }
    for pillar in branch_scores {
        if let Some(rows) = &pillar.rows {
            for row in rows {
                let Some(hidden_stem) = &row.hidden_stem else {
                    continue;
                };
                let element = if ["木", "火", "土", "金", "水"].contains(&hidden_stem.as_str())
                {
                    hidden_stem.clone()
                } else {
                    stem_element(hidden_stem)
                };
                if !element.is_empty() {
                    *totals.entry(element).or_insert(0.0) +=
                        row.final_contribution.unwrap_or(0.0).abs();
                }
            }
        }
    }
    totals
}

fn aggregate_cong_family_scores(
    stem_scores: &[QuantModelPillarScore],
    branch_scores: &[QuantModelPillarScore],
) -> (HashMap<&'static str, f64>, HashMap<&'static str, f64>) {
    let mut positive = HashMap::from([
        ("比劫", 0.0),
        ("印", 0.0),
        ("食傷", 0.0),
        ("財", 0.0),
        ("官殺", 0.0),
    ]);
    let mut negative = positive.clone();
    for score in stem_scores {
        if let Some(family) = family_of(&score.ten_god) {
            if score.final_score > 0.0 {
                *positive.entry(family).or_insert(0.0) += score.final_score;
            } else {
                *negative.entry(family).or_insert(0.0) += score.final_score.abs();
            }
        }
    }
    for pillar in branch_scores {
        if let Some(rows) = &pillar.rows {
            for row in rows {
                let Some(ten_god) = &row.ten_god else {
                    continue;
                };
                let Some(family) = family_of(ten_god) else {
                    continue;
                };
                let value = row.final_contribution.unwrap_or(0.0);
                if value > 0.0 {
                    *positive.entry(family).or_insert(0.0) += value;
                } else {
                    *negative.entry(family).or_insert(0.0) += value.abs();
                }
            }
        }
    }
    (positive, negative)
}

fn dominant_map_key<'a>(map: &'a HashMap<&'static str, f64>) -> Option<&'a str> {
    map.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
        .map(|(key, _)| *key)
}

fn find_dominant_trouble_family<'a>(
    family_scores: &'a HashMap<&'static str, f64>,
    unfavorable: &[&'a str],
) -> Option<&'a str> {
    unfavorable.iter().copied().max_by(|a, b| {
        family_scores
            .get(a)
            .copied()
            .unwrap_or(0.0)
            .abs()
            .partial_cmp(&family_scores.get(b).copied().unwrap_or(0.0).abs())
            .unwrap_or(Ordering::Equal)
    })
}

fn find_dominant_pressure_element(element_scores: &HashMap<String, f64>) -> Option<&str> {
    element_scores
        .iter()
        .find(|(_, value)| **value < -35.0)
        .map(|(key, _)| key.as_str())
}

fn family_of(ten_god: &str) -> Option<&'static str> {
    if ten_god.contains("印") {
        Some("印")
    } else if ten_god.contains('比') || ten_god.contains('劫') {
        Some("比劫")
    } else if ten_god.contains('食') || ten_god.contains('傷') {
        Some("食傷")
    } else if ten_god.contains('財') {
        Some("財")
    } else if ten_god.contains('官') || ten_god.contains('殺') {
        Some("官殺")
    } else {
        None
    }
}

fn family_label(family: &str) -> &str {
    family
}

fn generated_by_family(family: &str) -> &str {
    match family {
        "印" => "官殺",
        "比劫" => "印",
        "食傷" => "比劫",
        "財" => "食傷",
        "官殺" => "財",
        _ => "比劫",
    }
}

fn family_by_element<'a>(
    element: &str,
    peer_element: &'a str,
    print_element: &'a str,
    output_element: &'a str,
    wealth_element: &'a str,
    officer_element: &'a str,
) -> Option<&'a str> {
    if element == peer_element {
        Some("比劫")
    } else if element == print_element {
        Some("印")
    } else if element == output_element {
        Some("食傷")
    } else if element == wealth_element {
        Some("財")
    } else if element == officer_element {
        Some("官殺")
    } else {
        None
    }
}

fn family_elements(
    families: &[&str],
    peer_element: &str,
    print_element: &str,
    output_element: &str,
    wealth_element: &str,
    officer_element: &str,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for family in families {
        let element = match *family {
            "比劫" => peer_element,
            "印" => print_element,
            "食傷" => output_element,
            "財" => wealth_element,
            "官殺" => officer_element,
            _ => "",
        };
        if !element.is_empty() {
            seen.insert(element.to_string());
        }
    }
    seen.into_iter().collect()
}

fn mother_of(element: &str) -> String {
    match element {
        "木" => "水",
        "火" => "木",
        "土" => "火",
        "金" => "土",
        "水" => "金",
        _ => "",
    }
    .to_string()
}

fn child_of(element: &str) -> String {
    match element {
        "木" => "火",
        "火" => "土",
        "土" => "金",
        "金" => "水",
        "水" => "木",
        _ => "",
    }
    .to_string()
}

fn wealth_element_of(element: &str) -> String {
    match element {
        "木" => "土",
        "火" => "金",
        "土" => "水",
        "金" => "木",
        "水" => "火",
        _ => "",
    }
    .to_string()
}

fn officer_element_of(element: &str) -> String {
    match element {
        "木" => "金",
        "火" => "水",
        "土" => "木",
        "金" => "火",
        "水" => "土",
        _ => "",
    }
    .to_string()
}

fn generated_by_element(element: &str) -> String {
    mother_of(element)
}

fn elements_except(kept: &[&str]) -> Vec<String> {
    ["木", "火", "土", "金", "水"]
        .into_iter()
        .filter(|element| !kept.contains(element))
        .map(|element| element.to_string())
        .collect()
}

fn generates_element(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("木", "火") | ("火", "土") | ("土", "金") | ("金", "水") | ("水", "木")
    )
}

fn is_kill_god(ten_god: &str) -> bool {
    ten_god == "正官" || ten_god == "七殺" || ten_god == "官" || ten_god == "殺"
}

fn is_seal_god(ten_god: &str) -> bool {
    ten_god == "正印" || ten_god == "偏印" || ten_god == "印"
}

fn join_chinese(items: &[String]) -> String {
    if items.is_empty() {
        "無".to_string()
    } else {
        items.join("、")
    }
}

fn format_optional_score(value: Option<f64>) -> String {
    value
        .map(|item| format!("{:.1}", item))
        .unwrap_or_else(|| "-".to_string())
}

fn split_gan_zhi(value: &str) -> Option<(String, String)> {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() < 2 {
        None
    } else {
        Some((chars[0].to_string(), chars[1].to_string()))
    }
}

fn resolve_half_combination(a: &str, b: &str) -> Option<(String, String, String)> {
    match sorted_pair(a, b).as_str() {
        "午寅" | "午戌" => Some((
            "火".to_string(),
            "半合化火".to_string(),
            "半合化火".to_string(),
        )),
        "亥卯" | "卯未" => Some((
            "木".to_string(),
            "半合化木".to_string(),
            "半合化木".to_string(),
        )),
        "子申" | "子辰" => Some((
            "水".to_string(),
            "半合化水".to_string(),
            "半合化水".to_string(),
        )),
        "丑酉" | "巳酉" => Some((
            "金".to_string(),
            "半合化金".to_string(),
            "半合化金".to_string(),
        )),
        _ => None,
    }
}

fn resolve_three_combination(a: &str, b: &str, c: &str) -> Option<(String, String)> {
    match sorted_triplet(a, b, c).as_str() {
        "午寅戌" => Some(("火".to_string(), "寅午戌".to_string())),
        "亥卯未" => Some(("木".to_string(), "亥卯未".to_string())),
        "子申辰" => Some(("水".to_string(), "申子辰".to_string())),
        "丑巳酉" => Some(("金".to_string(), "巳酉丑".to_string())),
        _ => None,
    }
}

fn resolve_three_meeting(a: &str, b: &str, c: &str) -> Option<(String, String)> {
    match sorted_triplet(a, b, c).as_str() {
        "卯寅辰" => Some(("木".to_string(), "寅卯辰".to_string())),
        "午巳未" => Some(("火".to_string(), "巳午未".to_string())),
        "戌申酉" => Some(("金".to_string(), "申酉戌".to_string())),
        "丑亥子" => Some(("水".to_string(), "亥子丑".to_string())),
        _ => None,
    }
}

fn resolve_stem_combination(a: &str, b: &str) -> Option<(String, String)> {
    match sorted_pair(a, b).as_str() {
        "己甲" => Some(("甲己".to_string(), "土".to_string())),
        "乙庚" => Some(("乙庚".to_string(), "金".to_string())),
        "丙辛" => Some(("丙辛".to_string(), "水".to_string())),
        "丁壬" => Some(("丁壬".to_string(), "木".to_string())),
        "戊癸" => Some(("戊癸".to_string(), "火".to_string())),
        _ => None,
    }
}

fn sorted_pair(a: &str, b: &str) -> String {
    let mut items = [a.to_string(), b.to_string()];
    items.sort();
    items.join("")
}

fn sorted_triplet(a: &str, b: &str, c: &str) -> String {
    let mut items = [a.to_string(), b.to_string(), c.to_string()];
    items.sort();
    items.join("")
}

fn branch_name(branch: EarthBranch) -> String {
    trad(&branch.get_name())
}

fn stem_name(stem: HeavenStem) -> String {
    trad(&stem.get_name())
}

fn stem_element(stem: &str) -> String {
    match stem {
        "甲" | "乙" => "木",
        "丙" | "丁" => "火",
        "戊" | "己" => "土",
        "庚" | "辛" => "金",
        "壬" | "癸" => "水",
        _ => "",
    }
    .to_string()
}

fn position_weight(pillar: &str) -> f64 {
    match pillar {
        "年干" => 45.0,
        "月干" => 55.0,
        "時干" => 50.0,
        // 大運當值一字 60，歲運干支各 20（下冊 p197-198）。
        "運干" => 60.0,
        "流年干" => 20.0,
        "年支" => 45.0,
        "月支" => 100.0,
        "日支" => 55.0,
        "時支" => 50.0,
        "運支" => 60.0,
        "流年支" => 20.0,
        _ => 0.0,
    }
}

fn hidden_stems_of(branch: &str) -> Vec<String> {
    match branch {
        "子" => vec!["癸"],
        "丑" => vec!["己", "癸", "辛"],
        "寅" => vec!["甲", "丙", "戊"],
        "卯" => vec!["乙"],
        "辰" => vec!["戊", "乙", "癸"],
        "巳" => vec!["丙", "戊", "庚"],
        "午" => vec!["丁", "己"],
        "未" => vec!["己", "丁", "乙"],
        "申" => vec!["庚", "壬", "戊"],
        "酉" => vec!["辛"],
        "戌" => vec!["戊", "辛", "丁"],
        "亥" => vec!["壬", "甲"],
        _ => Vec::new(),
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn hidden_ratios(size: usize) -> Vec<f64> {
    match size {
        0 | 1 => vec![1.0],
        2 => vec![0.63, 0.37],
        _ => vec![0.5, 0.25, 0.25],
    }
}

fn resolve_ten_god(day_master_stem: &str, other_stem: &str) -> String {
    let dm = stem_element(day_master_stem);
    let other = stem_element(other_stem);
    let same_polarity = same_polarity(day_master_stem, other_stem);
    if dm == other {
        if same_polarity {
            "比肩"
        } else {
            "劫財"
        }
    } else if generates(&other, &dm) {
        if same_polarity {
            "偏印"
        } else {
            "正印"
        }
    } else if generates(&dm, &other) {
        if same_polarity {
            "食神"
        } else {
            "傷官"
        }
    } else if controls(&dm, &other) {
        if same_polarity {
            "偏財"
        } else {
            "正財"
        }
    } else if controls(&other, &dm) {
        if same_polarity {
            "七殺"
        } else {
            "正官"
        }
    } else {
        "未知"
    }
    .to_string()
}

fn resolve_element_ten_god(day_master_stem: &str, element: &str) -> String {
    let dm = stem_element(day_master_stem);
    if dm == element {
        "比肩"
    } else if generates(element, &dm) {
        "正印"
    } else if generates(&dm, element) {
        "食神"
    } else if controls(&dm, element) {
        "正財"
    } else if controls(element, &dm) {
        "正官"
    } else {
        "未知"
    }
    .to_string()
}

fn resolve_ten_god_score(ten_god: &str) -> f64 {
    match ten_god {
        "比肩" | "劫財" => 100.0,
        "正印" | "偏印" => 80.0,
        "食神" | "傷官" => -45.0,
        "正財" | "偏財" => -60.0,
        "正官" | "七殺" => -75.0,
        _ => 0.0,
    }
}

fn resolve_element_score(day_master_stem: &str, element: &str) -> f64 {
    resolve_ten_god_score(&resolve_element_ten_god(day_master_stem, element))
}

fn resolve_branch_hidden_stem_score(
    _day_master_stem: &str,
    _branch: &str,
    _hidden_stem: &str,
    ten_god: &str,
) -> f64 {
    resolve_ten_god_score(ten_god)
}

fn stem_for_element(element: &str, yang: bool) -> String {
    match element {
        "木" => {
            if yang {
                "甲"
            } else {
                "乙"
            }
        }
        "火" => {
            if yang {
                "丙"
            } else {
                "丁"
            }
        }
        "土" => {
            if yang {
                "戊"
            } else {
                "己"
            }
        }
        "金" => {
            if yang {
                "庚"
            } else {
                "辛"
            }
        }
        "水" => {
            if yang {
                "壬"
            } else {
                "癸"
            }
        }
        _ => "",
    }
    .to_string()
}

fn is_element_lu_branch(day_master: &str, branch: &str) -> bool {
    match stem_element(day_master).as_str() {
        "木" => ["寅", "卯"].contains(&branch),
        "火" => ["巳", "午"].contains(&branch),
        "土" => ["巳", "午"].contains(&branch),
        "金" => ["申", "酉"].contains(&branch),
        "水" => ["亥", "子"].contains(&branch),
        _ => false,
    }
}

fn same_polarity(a: &str, b: &str) -> bool {
    is_yang(a) == is_yang(b)
}

fn is_yang(stem: &str) -> bool {
    matches!(stem, "甲" | "丙" | "戊" | "庚" | "壬")
}

fn generates(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("木", "火") | ("火", "土") | ("土", "金") | ("金", "水") | ("水", "木")
    )
}

fn controls(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("木", "土") | ("火", "金") | ("土", "水") | ("金", "木") | ("水", "火")
    )
}

fn distance(a: usize, b: usize) -> usize {
    a.abs_diff(b)
}

/// 大運、歲運游走於四柱之間（《量化五行力》下冊 p202）。
fn is_floating_pillar(pillar: &str) -> bool {
    matches!(pillar, "運干" | "運支" | "流年干" | "流年支")
}

/// 干支之間的沖合距離：大運、歲運入局一律以緊貼論。
fn interaction_distance(left_pillar: &str, left: usize, right_pillar: &str, right: usize) -> usize {
    if is_floating_pillar(left_pillar) || is_floating_pillar(right_pillar) {
        1
    } else {
        distance(left, right)
    }
}

fn branch_pair_distance(left: &BranchInput, right: &BranchInput) -> usize {
    interaction_distance(&left.pillar, left.position_index, &right.pillar, right.position_index)
}

fn stem_pair_distance(left: &StemInput, right: &StemInput) -> usize {
    interaction_distance(&left.pillar, left.position_index, &right.pillar, right.position_index)
}

/// 得根、透干距離：大運、歲運入局一律以同柱論。
fn stem_branch_distance(stem: &StemInput, branch: &BranchInput) -> usize {
    if is_floating_pillar(&stem.pillar) || is_floating_pillar(&branch.pillar) {
        0
    } else {
        distance(stem.position_index, branch.position_index)
    }
}

fn normalize_distance(distance: usize) -> usize {
    distance.min(3)
}

fn resolve_branch_element(branch: &str) -> String {
    match branch {
        "寅" | "卯" => "木",
        "巳" | "午" => "火",
        "申" | "酉" => "金",
        "亥" | "子" => "水",
        "辰" | "戌" | "丑" | "未" => "土",
        _ => "",
    }
    .to_string()
}

fn is_six_combination(a: &str, b: &str) -> bool {
    !resolve_six_combination_element(a, b).is_empty()
}

fn resolve_six_combination_element(a: &str, b: &str) -> String {
    match (a, b) {
        ("子", "丑") | ("丑", "子") => "土",
        ("寅", "亥") | ("亥", "寅") => "木",
        ("卯", "戌") | ("戌", "卯") => "火",
        ("辰", "酉") | ("酉", "辰") => "金",
        ("巳", "申") | ("申", "巳") => "水",
        ("午", "未") | ("未", "午") => "土",
        _ => "",
    }
    .to_string()
}

fn resolve_six_combination_factor(branch: &BranchInput, element: &str) -> f64 {
    if is_six_combination_specialist(&branch.branch, element) {
        1.0
    } else {
        0.8
    }
}

fn resolve_six_combination_residual_factor(branch: &BranchInput, element: &str) -> f64 {
    if is_six_combination_specialist(&branch.branch, element) {
        0.0
    } else {
        0.2
    }
}

fn resolve_half_combination_factor(
    candidate: &HalfCombinationCandidate,
    branch: &BranchInput,
) -> f64 {
    if is_half_combination_specialist(&branch.branch, &candidate.element) {
        match normalize_distance(candidate.distance) {
            1 => 1.2,
            2 => 1.1,
            _ => 1.0,
        }
    } else {
        match normalize_distance(candidate.distance) {
            1 => 0.8,
            2 => 0.7,
            _ => 0.5,
        }
    }
}

fn resolve_half_combination_residual_factor(
    candidate: &HalfCombinationCandidate,
    branch: &BranchInput,
) -> f64 {
    if is_half_combination_specialist(&branch.branch, &candidate.element) {
        0.0
    } else {
        match normalize_distance(candidate.distance) {
            1 => 0.2,
            2 => 0.3,
            _ => 0.5,
        }
    }
}

/// 三合化：專氣支 1.4，其他二支 1.02，化後沒有餘氣（上冊 p132）。
fn resolve_three_combination_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    if role == CombinationBranchRole::Specialist {
        1.4
    } else {
        1.02
    }
}

fn resolve_three_combination_residual_factor(_label: &str, _role: CombinationBranchRole) -> f64 {
    0.0
}

/// 三會化：專氣支及長生支 2.1，墓庫支 1.05，化後沒有餘氣（上冊 p132）。
fn resolve_three_meeting_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    match role {
        CombinationBranchRole::Specialist | CombinationBranchRole::Growth => 2.1,
        CombinationBranchRole::Storage => 1.05,
        CombinationBranchRole::Other => 1.0,
    }
}

fn resolve_three_meeting_residual_factor(_label: &str, _role: CombinationBranchRole) -> f64 {
    0.0
}

fn is_six_combination_specialist(branch: &str, target_element: &str) -> bool {
    resolve_branch_element(branch) == target_element
}

fn is_half_combination_specialist(branch: &str, target_element: &str) -> bool {
    resolve_branch_element(branch) == target_element
}

fn is_specialist_branch(branch: &str) -> bool {
    matches!(branch, "子" | "午" | "卯" | "酉")
}

fn is_stem_clash(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("甲", "庚")
            | ("庚", "甲")
            | ("乙", "辛")
            | ("辛", "乙")
            | ("丙", "壬")
            | ("壬", "丙")
            | ("丁", "癸")
            | ("癸", "丁")
    )
}

/// 天干隔一位相沖採簡化法（上冊 p147）：只計緊貼沖，隔一位不計。
/// 改成 false 即回到精算法（下冊 p148：隔一位各 ×0.875，合而不化又被隔位沖 ×0.645）。
const STEM_CLASH_SIMPLIFIED: bool = true;

fn is_supported_stem_clash_distance(dist: usize) -> bool {
    dist == 1 || (!STEM_CLASH_SIMPLIFIED && dist == 2)
}

fn stem_clash_remainder(dist: usize) -> f64 {
    match dist {
        1 => 0.5,
        2 => 0.875,
        _ => 1.0,
    }
}

fn is_direct_clash(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("子", "午")
            | ("午", "子")
            | ("丑", "未")
            | ("未", "丑")
            | ("寅", "申")
            | ("申", "寅")
            | ("卯", "酉")
            | ("酉", "卯")
            | ("辰", "戌")
            | ("戌", "辰")
            | ("巳", "亥")
            | ("亥", "巳")
    )
}

fn classify_branch_clash(a: &str, b: &str) -> BranchClashType {
    if matches!(
        (a, b),
        ("子", "午") | ("午", "子") | ("卯", "酉") | ("酉", "卯")
    ) {
        BranchClashType::Specialist
    } else if matches!(
        (a, b),
        ("寅", "申") | ("申", "寅") | ("巳", "亥") | ("亥", "巳")
    ) {
        BranchClashType::Growth
    } else {
        BranchClashType::Storage
    }
}

/// 地支相沖的耗損一律由支氣折減（BranchQiModifier）處理，這裡不再整支相乘，
/// 避免專氣沖被扣兩次（上冊 p45：隔一位相沖只 ×0.75）。
fn branch_clash_score_multiplier(_clash_type: BranchClashType, _dist: usize) -> f64 {
    1.0
}

/// 受沖支氣餘量：緊貼化零、隔一位 ×0.75、隔二位不計（上冊 p45-46、p147）。
fn branch_clash_affected_multiplier(dist: usize) -> f64 {
    match dist {
        1 => 0.0,
        2 => 0.75,
        _ => 1.0,
    }
}

fn branch_clash_note(clash_type: BranchClashType, dist: usize) -> String {
    match clash_type {
        BranchClashType::Specialist => match dist {
            1 => "近沖".to_string(),
            2 => "隔一位沖".to_string(),
            _ => "隔二位沖（不計）".to_string(),
        },
        BranchClashType::Growth => match dist {
            1 => "近沖（長生受沖支氣化零）".to_string(),
            2 => "隔一位沖（長生受沖支氣×0.75）".to_string(),
            _ => "隔二位沖（不計）".to_string(),
        },
        BranchClashType::Storage => match dist {
            1 => "近沖（墓庫旁氣化零）".to_string(),
            2 => "隔一位沖（墓庫旁氣×0.75）".to_string(),
            _ => "隔二位沖（不計）".to_string(),
        },
    }
}

fn clash_type_distance_note(prefix: &str, _dist: usize, affected_multiplier: f64) -> String {
    match affected_multiplier {
        value if value <= 0.0 => format!("{}：受沖支氣化零，未受沖主氣不變", prefix),
        _ => format!(
            "{}：受沖支氣×{}，未受沖主氣不變",
            prefix,
            round3_string(affected_multiplier)
        ),
    }
}

fn branch_role(label: &str, branch: &str) -> CombinationBranchRole {
    match label {
        "申子辰" | "亥子丑" => {
            if branch == "子" {
                CombinationBranchRole::Specialist
            } else if branch == "申" || branch == "亥" {
                CombinationBranchRole::Growth
            } else {
                CombinationBranchRole::Storage
            }
        }
        "寅午戌" | "巳午未" => {
            if branch == "午" {
                CombinationBranchRole::Specialist
            } else if branch == "寅" || branch == "巳" {
                CombinationBranchRole::Growth
            } else {
                CombinationBranchRole::Storage
            }
        }
        "亥卯未" | "寅卯辰" => {
            if branch == "卯" {
                CombinationBranchRole::Specialist
            } else if branch == "亥" || branch == "寅" {
                CombinationBranchRole::Growth
            } else {
                CombinationBranchRole::Storage
            }
        }
        "巳酉丑" | "申酉戌" => {
            if branch == "酉" {
                CombinationBranchRole::Specialist
            } else if branch == "巳" || branch == "申" {
                CombinationBranchRole::Growth
            } else {
                CombinationBranchRole::Storage
            }
        }
        _ => CombinationBranchRole::Other,
    }
}

fn transparency_multiplier(dist: usize) -> f64 {
    match normalize_distance(dist) {
        0 => 0.5,
        1 => 0.4,
        2 => 0.3,
        _ => 0.2,
    }
}

fn root_multiplier(root_level: RootLevel, dist: usize) -> f64 {
    match root_level {
        RootLevel::Lu => match normalize_distance(dist) {
            0 => 0.5,
            1 => 0.4,
            2 => 0.3,
            _ => 0.2,
        },
        // 中根：亥中甲、午中己，0.5 × 0.37 = 0.185（上冊 p29-30）。
        RootLevel::Middle => match normalize_distance(dist) {
            0 => 0.185,
            1 => 0.148,
            2 => 0.111,
            _ => 0.074,
        },
        // 墓庫根：戊己得辰戌丑未主氣，0.5 × 0.5 = 0.25（上冊 p30）。
        RootLevel::Tomb => match normalize_distance(dist) {
            0 => 0.25,
            1 => 0.2,
            2 => 0.15,
            _ => 0.1,
        },
        RootLevel::Minor => match normalize_distance(dist) {
            0 => 0.125,
            1 => 0.1,
            2 => 0.075,
            _ => 0.05,
        },
        RootLevel::None => 0.0,
    }
}

fn branch_qi_multiplier(modifier: Option<&BranchQiModifier>, stem: &str) -> f64 {
    let Some(modifier) = modifier else {
        return 1.0;
    };
    if let Some(value) = modifier.stem_multipliers.get(stem) {
        *value
    } else {
        modifier
            .element_multipliers
            .get(&stem_element(stem))
            .copied()
            .unwrap_or(1.0)
    }
}

fn branch_score_qi_multiplier(
    _branch: &BranchInput,
    stem: &str,
    branch_interaction: &BranchInteraction,
    modifier: Option<&BranchQiModifier>,
) -> f64 {
    let _ = branch_interaction;
    branch_qi_multiplier(modifier, stem)
}

fn format_branch_pair(left_index: usize, right_index: usize, branches: &[BranchInput]) -> String {
    let left = &branches[left_index.min(right_index)];
    let right = &branches[left_index.max(right_index)];
    format!("{}{}", left.branch, right.branch)
}

fn resolve_month_main_stem(month_branch: &BranchInput) -> String {
    if month_branch.hidden_stems.is_empty() {
        hidden_stems_of(&month_branch.branch)
            .into_iter()
            .next()
            .unwrap_or_default()
    } else {
        month_branch
            .hidden_stems
            .first()
            .cloned()
            .unwrap_or_default()
    }
}

/// 天干合化需月令有化神（主氣或旁氣均可），且月令主氣不剋化神（上冊 p99、p101）。
/// 月令若半合或六合化，化前支氣仍可作化神，但剋制與否改看化後五行；
/// 月令若三合或三會化，已無餘氣，只看化後五行。
fn month_allows_stem_transformation(
    target_element: &str,
    month_branch: &BranchInput,
    month_transform: Option<&BranchTransform>,
) -> bool {
    let (has_transforming_qi, controlling_element) = match month_transform {
        Some(transform)
            if matches!(
                transform.category,
                BranchTransformCategory::ThreeCombination | BranchTransformCategory::ThreeMeeting
            ) =>
        {
            (transform.element == target_element, transform.element.clone())
        }
        Some(transform) => (
            transform.element == target_element
                || month_hidden_elements(month_branch).contains(&target_element.to_string()),
            transform.element.clone(),
        ),
        None => (
            month_hidden_elements(month_branch).contains(&target_element.to_string()),
            stem_element(&resolve_month_main_stem(month_branch)),
        ),
    };
    has_transforming_qi && !controls(&controlling_element, target_element)
}

fn month_hidden_elements(month_branch: &BranchInput) -> Vec<String> {
    let hidden_stems = if month_branch.hidden_stems.is_empty() {
        hidden_stems_of(&month_branch.branch)
    } else {
        month_branch.hidden_stems.clone()
    };
    hidden_stems.iter().map(|stem| stem_element(stem)).collect()
}

fn is_chen_xu_pair(a: &str, b: &str) -> bool {
    matches!((a, b), ("辰", "戌") | ("戌", "辰"))
}

fn primary_hidden_stem(branch: &BranchInput) -> String {
    if branch.hidden_stems.is_empty() {
        hidden_stems_of(&branch.branch)
            .into_iter()
            .next()
            .unwrap_or_default()
    } else {
        branch.hidden_stems.first().cloned().unwrap_or_default()
    }
}

fn primary_hidden_stem_ratio(branch: &BranchInput) -> f64 {
    let count = if branch.hidden_stems.is_empty() {
        hidden_stems_of(&branch.branch).len()
    } else {
        branch.hidden_stems.len()
    };
    hidden_ratios(count).first().copied().unwrap_or(1.0)
}

fn branch_prune_blocks_stem(effect: Option<&BranchPruneEffect>, hidden_stem: &str) -> bool {
    effect.is_some_and(|item| item.remaining_stem != hidden_stem)
}

fn stacked_six_combination_power_for_pair(
    input: &BranchInput,
    other: &BranchInput,
    branches: &[BranchInput],
) -> f64 {
    if is_stacked_six_combination_center(input, branches)
        || is_stacked_six_combination_center(other, branches)
    {
        2.0 * 0.96
    } else {
        0.0
    }
}

fn is_stacked_six_combination_center(branch: &BranchInput, branches: &[BranchInput]) -> bool {
    let index = branch.position_index;
    if index == 0 || index >= branches.len() - 1 || is_floating_pillar(&branch.pillar) {
        return false;
    }
    let left = &branches[index - 1];
    let right = &branches[index + 1];
    if is_floating_pillar(&left.pillar) || is_floating_pillar(&right.pillar) {
        return false;
    }
    is_six_combination(&left.branch, &branch.branch)
        && is_six_combination(&branch.branch, &right.branch)
        && resolve_six_combination_element(&left.branch, &branch.branch)
            == resolve_six_combination_element(&branch.branch, &right.branch)
}

fn half_combination_power(candidate: &HalfCombinationCandidate) -> f64 {
    match normalize_distance(candidate.distance) {
        1 => 2.05,
        2 => 1.88,
        _ => 1.625,
    }
}

fn lu_root_branches(element: &str) -> &'static [&'static str] {
    match element {
        "木" => &["寅", "卯"],
        "火" => &["巳", "午"],
        "金" => &["申", "酉"],
        "水" => &["亥", "子"],
        _ => &[],
    }
}

fn middle_root_branches(element: &str) -> &'static [&'static str] {
    match element {
        "木" => &["亥"],
        "土" => &["午"],
        _ => &[],
    }
}

fn minor_root_branches(element: &str) -> &'static [&'static str] {
    match element {
        "木" => &["寅", "未"],
        "火" => &["寅", "未"],
        "土" => &["寅", "巳"],
        "金" => &["巳", "戌", "申"],
        "水" => &["申", "丑", "子"],
        _ => &[],
    }
}

fn tomb_root_branches(element: &str) -> &'static [&'static str] {
    match element {
        "土" => &["辰", "戌", "丑", "未"],
        _ => &[],
    }
}

const COND_FOLLOW_BOUNDARY: f64 = 210.0;

/// 「傾向」要看身強身弱：分數正＝幫日主（印比）、負＝剋洩耗。正格身強喜負分（剋洩耗），
/// 正格身弱喜正分（扶身）；從強（總分 ≥ +210）順印比之勢喜正分，從弱（≤ −210）順食財官之勢喜負分
/// （第十篇〈從格〉臨界）。等於看運歲把作用後命局推向 0（中和）還是推離 0。
fn tendency_label(score: f64, natal_total: f64) -> String {
    let prefer_positive = natal_total >= COND_FOLLOW_BOUNDARY
        || (natal_total < 0.0 && natal_total > -COND_FOLLOW_BOUNDARY);
    let score = if prefer_positive { score } else { -score };
    if score >= 8.0 {
        "偏吉".to_string()
    } else if score >= 2.0 {
        "可用".to_string()
    } else if score <= -8.0 {
        "偏壓".to_string()
    } else if score <= -2.0 {
        "需防".to_string()
    } else {
        "平".to_string()
    }
}

fn element_relation_score(
    day_element: tyme4rs::tyme::culture::Element,
    target_element: tyme4rs::tyme::culture::Element,
) -> f64 {
    if day_element == target_element {
        1.0
    } else if (target_element.get_index() + 1) % 5 == day_element.get_index() {
        1.1
    } else if (day_element.get_index() + 1) % 5 == target_element.get_index() {
        -0.8
    } else if (day_element.get_index() + 2) % 5 == target_element.get_index() {
        -1.0
    } else if (target_element.get_index() + 2) % 5 == day_element.get_index() {
        -1.15
    } else {
        0.0
    }
}

fn trad(value: &str) -> String {
    value
        .replace('阳', "陽")
        .replace('阴', "陰")
        .replace('龙', "龍")
        .replace('惊', "驚")
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn round3_string(value: f64) -> String {
    let s = format!("{:.3}", round3(value));
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod book_tests {
    //! 《八字算命之量化五行力》上下冊的書上案例與規則回歸。
    use super::*;

    /// 四柱依年、月、日、時輸入，如 ["癸未", "壬戌", "乙丑", "丙子"]。
    fn chart(pillars: [&str; 4]) -> (String, CalculationInputs) {
        let stem = |index: usize| pillars[index].chars().next().unwrap().to_string();
        let branch = |index: usize| pillars[index].chars().nth(1).unwrap().to_string();
        let stems = vec![
            StemInput { pillar: "年干".into(), stem: stem(0), position_index: 0 },
            StemInput { pillar: "月干".into(), stem: stem(1), position_index: 1 },
            StemInput { pillar: "時干".into(), stem: stem(3), position_index: 3 },
        ];
        let branches = ["年支", "月支", "日支", "時支"]
            .iter()
            .enumerate()
            .map(|(index, pillar)| BranchInput {
                pillar: pillar.to_string(),
                branch: branch(index),
                hidden_stems: hidden_stems_of(&branch(index)),
                position_index: index,
            })
            .collect();
        (stem(2), CalculationInputs { stems, branches })
    }

    fn scores(pillars: [&str; 4]) -> HashMap<String, f64> {
        let (day_master, inputs) = chart(pillars);
        let bundle = score_resolved_chart(&day_master, &inputs);
        let mut map: HashMap<String, f64> = bundle
            .stem_scores
            .iter()
            .chain(bundle.branch_scores.iter())
            .map(|item| (item.pillar.clone(), item.final_score))
            .collect();
        let total = round1(map.values().sum());
        map.insert("總分".into(), total);
        map
    }

    fn assert_score(map: &HashMap<String, f64>, pillar: &str, expected: f64) {
        let actual = map[pillar];
        assert!(
            (actual - expected).abs() < 0.051,
            "{} expected {:+.1}, got {:+.1}",
            pillar,
            expected,
            actual
        );
    }

    #[test]
    fn s_case_matches_book() {
        // 上冊 p52-54 S先生，簡化法略去丙壬隔一位沖，總分 +24。
        let map = scores(["癸未", "壬戌", "乙丑", "丙子"]);
        assert_score(&map, "年干", 41.1);
        assert_score(&map, "月干", 53.2);
        assert_score(&map, "總分", 24.1);
    }

    #[test]
    fn special_male_case_matches_book() {
        // 下冊第七篇特殊命例：巳午未三會不化。書上用精算法（甲庚隔一位 ×0.875）得 +14.5；
        // 本程式採簡化法略去隔一位干沖，地支各列仍與書相同。
        let map = scores(["癸巳", "甲子", "己未", "庚午"]);
        assert_score(&map, "年干", -37.8);
        assert_score(&map, "月支", -84.0);
        assert_score(&map, "年支", 78.2);
        assert_score(&map, "時支", 82.8);
        assert_score(&map, "總分", 5.3);
    }

    #[test]
    fn clashed_lu_branch_keeps_only_remaining_hidden_stems() {
        // 八字真理上冊 p71 阮玲玉：己日巳月被亥緊貼沖，巳只剩戊 +25。
        let map = scores(["庚戌", "辛巳", "己亥", "乙亥"]);
        assert_score(&map, "月支", 25.0);
    }

    #[test]
    fn three_meeting_boost_is_not_repeated_by_second_wei() {
        // 八字真理上冊 p52 柯震東（己巳時）：兩未各與巳午成三會不化，巳午只提升一次。
        let map = scores(["辛未", "甲午", "己未", "己巳"]);
        assert_score(&map, "年支", 33.2);
        assert_score(&map, "日支", 46.1);
        assert_score(&map, "時支", 93.2);
    }

    #[test]
    fn shen_you_xu_and_hai_zi_chou_meetings_are_detected() {
        // 八字真理下冊 p211：亥子丑三會化水，總分 +380。
        let map = scores(["丙子", "己亥", "乙丑", "壬午"]);
        assert_score(&map, "總分", 380.1);
        // 下冊 p222 鄧太：申酉戌三會不化，午戌半合不成立。
        let map = scores(["丙申", "戊戌", "癸酉", "戊午"]);
        assert_score(&map, "時支", -47.8);
        assert_score(&map, "總分", -129.9);
    }

    #[test]
    fn yin_shen_clash_case_matches_book() {
        // 下冊第七篇案例一：寅申近沖，總分 +9（天干 +81.1、月令 -18.8）。
        let map = scores(["壬申", "壬寅", "壬午", "甲辰"]);
        assert_score(&map, "月支", -18.8);
        assert_score(&map, "總分", 9.4);
    }

    #[test]
    fn middle_root_uses_0_185() {
        // 上冊 p32：時干戊得中根在年支午、墓庫根在月令丑、微根在日支申，得 +53。
        let map = scores(["辛午", "辛丑", "庚申", "戊卯"]);
        assert_score(&map, "時干", 53.0);
    }

    #[test]
    fn hidden_stem_transparency_example() {
        // 上冊 p34：日支巳藏丙戊庚，各自透干，得 +4.5。
        let map = scores(["辛卯", "戊辰", "辛巳", "丙寅"]);
        assert_score(&map, "日支", 4.5);
    }

    #[test]
    fn lu_branch_keeps_hidden_stem_transparency() {
        // 上冊 p35：月令寅為乙之祿，+100 再加甲丙戊透干增值，得 +103.3。
        let map = scores(["己卯", "丙寅", "乙酉", "甲子"]);
        assert_score(&map, "月支", 103.3);
    }

    #[test]
    fn separated_specialist_clash_is_counted_once() {
        // 上冊 p45：子午隔一位相沖，二支各 ×0.75（不是 ×0.75×0.75）。
        let map = scores(["壬子", "乙卯", "庚午", "丁丑"]);
        assert_score(&map, "年支", -22.8);
    }

    #[test]
    fn clashed_growth_branch_loses_qi_regardless_of_polarity() {
        // 量化五行力不分陰陽（p20）；寅申緊貼沖，寅中木火化零（p45），乙在寅不再得根。
        let (day_master, inputs) = chart(["乙丑", "戊寅", "壬申", "辛丑"]);
        let bundle = score_resolved_chart(&day_master, &inputs);
        let year_stem = &bundle.stem_scores[0];
        assert_eq!(year_stem.bonus_score, Some(0.0), "{}", year_stem.details);
        // 乙日主月令寅被申緊貼沖：祿氣化零，只剩戊土旁氣 -60 × 0.25。
        let map = scores(["壬申", "壬寅", "乙酉", "丁丑"]);
        assert_score(&map, "月支", -15.0);
    }

    #[test]
    fn six_combination_cushions_specialist_clash() {
        // 上冊 p139：卯酉沖遇辰酉合，以沖論，卯酉各 ×0.48，辰不變。
        let map = scores(["癸辰", "丁酉", "癸卯", "己未"]);
        assert_score(&map, "月支", 38.4);
        assert_score(&map, "日支", -11.9);
    }

    #[test]
    fn pinched_specialist_clash_halves_outer_branches() {
        // 上冊 p145：酉受二卯夾沖，酉化零，二卯各 ×0.5。
        let map = scores(["甲子", "乙卯", "丙酉", "己卯"]);
        assert_score(&map, "日支", 0.0);
        assert_score(&map, "月支", 76.0);
    }

    #[test]
    fn separated_six_combination_is_ignored() {
        // 上冊 p46、p128：六合隔一位不需理會。
        let (day_master, inputs) = chart(["甲子", "丙寅", "己丑", "丙寅"]);
        let bundle = score_resolved_chart(&day_master, &inputs);
        assert_eq!(bundle.branch_scores[0].clash_adjustment, Some(0.0));
        assert_eq!(bundle.branch_scores[2].clash_adjustment, Some(0.0));
    }

    #[test]
    fn stem_combination_can_use_month_side_qi_as_transforming_god() {
        // 上冊 p99：月令主氣或旁氣有化神即可；亥中甲木為化神，壬水不剋木，丁壬合化木。
        let (day_master, inputs) = chart(["丁亥", "壬亥", "丙戌", "己丑"]);
        let state = resolve_chart_state(&day_master, &inputs);
        assert!(state.stem_combination_effects[&0].transforms);
    }

    #[test]
    fn luck_pillar_enters_beside_every_natal_pillar() {
        // 下冊 p202：大運入局得根透干按同柱、沖合按緊貼。
        let (day_master, natal) = chart(["甲子", "丁卯", "己巳", "丙寅"]);
        let inputs = build_extended_inputs(&natal, Some("庚"), None, None, None);
        let state = resolve_chart_state(&day_master, &inputs);
        assert_eq!(state.stem_clash_effects[&0].remainder, 0.5);
        let luck = inputs.stems.iter().find(|item| item.pillar == "運干").unwrap();
        assert_eq!(stem_branch_distance(luck, &inputs.branches[0]), 0);
        assert_ne!(luck.position_index, 3);
    }

    #[test]
    fn luck_impact_uses_relative_formula() {
        // 下冊 p199-200：M=+100、L=-50 → R=-0.45、M′=+55；M=-100、L=-50 → M′=-145。
        assert_eq!(luck_impact_ratio(-50.0, 100.0), -0.45);
        assert_eq!(luck_effective_natal_score(100.0, -0.45), 55.0);
        assert_eq!(luck_effective_natal_score(-100.0, -0.45), -145.0);
        // 大運不能使命局翻號。
        let ratio = luck_impact_ratio(-500.0, 10.0);
        assert!(luck_effective_natal_score(10.0, ratio) >= 0.0);
    }
    fn luck_overview(pillars: [&str; 4], gan_zhi: &str) -> QuantLuckScore {
        let (day_master, natal) = chart(pillars);
        let natal_total = natal_cells_total(&score_resolved_chart(&day_master, &natal));
        let da_yun = DaYunDto {
            index: 1,
            start_age: None,
            end_age: None,
            start_year: None,
            end_year: None,
            gan_zhi: gan_zhi.into(),
            stem_ten_god: String::new(),
            branch_hidden_stems: vec![],
            branch_ten_gods: vec![],
            xun: String::new(),
            xun_kong: String::new(),
            liu_nian: vec![],
        };
        build_luck_scores(&day_master, natal_total, &natal, &[da_yun]).remove(0)
    }

    #[test]
    fn luck_impact_includes_natal_change() {
        // 八字真理上冊 p70：己運 −45，合甲令原局 −77，整體衝擊 −122；M=+100 時 R 由 0.41 變 0.77。
        assert_eq!(luck_impact_ratio(-45.0, 100.0), -0.41);
        assert_eq!(luck_impact_ratio(-122.0, 100.0), -0.77);
        let luck = luck_overview(["戊子", "丙申", "丙辰", "庚寅"], "辛卯");
        let lines = luck.overview_lines.unwrap();
        assert!(lines.iter().any(|line| line.starts_with("運干入局令原局變化 -200.7")));
        // 運干 −77.3 與原局 −200.7 合計 −278，運支 +69.6 與 +63.3 合計 +132.9，平均 −72.6。
        assert_eq!(luck.impact_ratio, luck_impact_ratio(-72.6, luck_natal_total(["戊子", "丙申", "丙辰", "庚寅"])));
    }

    fn luck_natal_total(pillars: [&str; 4]) -> f64 {
        let (day_master, natal) = chart(pillars);
        natal_cells_total(&score_resolved_chart(&day_master, &natal))
    }

    #[test]
    fn wandering_luck_notes_clash_time_share_and_held_combination() {
        // 八字真理上冊 p58-60：酉運沖月支卯，約 3/4 時間化零、1/4 時間隔一位 ×0.75；
        // 運干與局中天干合而不化，被合住在該位置。
        let clash = luck_overview(["戊子", "甲卯", "丙辰", "庚寅"], "辛酉");
        assert!(clash.overview_lines.unwrap().iter().any(|line| line
            .contains("運支酉與月支卯相沖，約 3/4 時間緊貼化零、約 1/4 時間隔一位剩 0.75")));
        let held = luck_overview(["戊子", "甲戌", "丙午", "庚寅"], "己酉");
        assert!(held
            .overview_lines
            .unwrap()
            .iter()
            .any(|line| line.contains("運干己與月干甲合而不化，被合住在該位置")));
        assert_eq!(
            wandering_clash_line("運支", "午", "時支", "子", 3),
            "游走的大運：運支午與時支子相沖，約 2/4 時間緊貼化零、約 1/4 時間隔一位剩 0.75、約 1/4 時間隔二位剩 8/9；平均約剩 0.41（計分仍按緊貼）"
        );
    }
    #[test]
    fn six_combination_cushions_growth_and_storage_clash() {
        // 量化五行力下冊 p190-191 B女士：寅亥六合不化緩沖巳亥沖，亥、巳受沖之氣 ×0.64，書 −9。
        assert_score(&scores(["丙寅", "己亥", "辛巳", "辛卯"]), "總分", -8.6);
        // 八字真理上冊 p136 胡適：丑未沖遇子丑合，合可緩沖，幾近以無沖無合推算，書 −67。
        assert_score(&scores(["辛卯", "庚子", "丁丑", "丁未"]), "總分", -67.6);
        // 上冊 p30 沖剋男命：寅亥合的兩支都被沖，以沖不以合論；巳只剩戊 +25、申只剩戊 +13.8。
        let map = scores(["乙亥", "辛巳", "戊申", "甲寅"]);
        assert_score(&map, "月支", 25.0);
        assert_score(&map, "日支", 13.8);
    }

    #[test]
    fn sandwiched_six_combination_follows_book() {
        // 量化五行力下冊第七篇假從女命：未午未夾合不化，午只餘 0.04，書 −63。
        assert_score(&scores(["丁未", "丙午", "辛未", "庚寅"]), "總分", -65.1);
        // 八字真理上冊 p52 柯震東戊辰時：午只和靠近化神（時干戊）的日未化土，年未不變，書 +266。
        assert_score(&scores(["辛未", "甲午", "己未", "戊辰"]), "總分", 273.4);
        // 下冊 p257 例二：二寅夾合亥，互相牽制，全部合而不化，書 −109。
        assert_score(&scores(["癸卯", "甲寅", "己亥", "丙寅"]), "總分", -108.5);
    }

    #[test]
    fn damaged_month_or_strong_stems_change_transformation() {
        // 八字真理下冊 p238：卯戌六合不化傷了月令戌，月令擋不住子辰半合化水，書 +32。
        assert_score(&scores(["辛卯", "戊戌", "壬子", "甲辰"]), "總分", 31.4);
        // 下冊 p222 阿宏：年月丙丁剋金，辰酉、酉丑都不化，書 −201。
        assert_score(&scores(["丙辰", "丁酉", "乙丑", "庚辰"]), "總分", -200.8);
    }

    #[test]
    fn four_storage_gathering_transforms_or_boosts_earth() {
        // 八字真理上冊 p55：辰戌丑未齊全、天干見土，合化巨土，各支 1.15 個土。
        let map = scores(["戊辰", "乙丑", "甲戌", "辛未"]);
        assert_score(&map, "年支", -46.7);
        assert_score(&map, "總分", -286.9);
        // 天干不見土：合而不化，各支土氣由 0.5 提升到 0.7。
        let map = scores(["壬辰", "癸丑", "甲戌", "乙未"]);
        assert_score(&map, "總分", 80.1);
    }

    #[test]
    fn two_clashers_share_force_only_for_lu_branches() {
        // 八字真理下冊 p250：二巳沖一亥，月巳擔八成沖力、年巳擔二成；年巳 +38.6、酉 −30 與書相同。
        // 月巳書 +4.9、程式 −5.2，總分書 +9、程式 −1.7（月巳藏干的獨立推算細節書上沒列出）。
        let map = scores(["癸巳", "丁巳", "丁亥", "己酉"]);
        assert_score(&map, "年支", 38.6);
        assert_score(&map, "時支", -30.0);
        // 上冊 p161 李國寶：二卯沖一酉，年卯隔一位不受影響，書 +49。
        assert_score(&scores(["己卯", "丁卯", "己酉", "甲戌"]), "總分", 48.6);
        // 上冊 p71 阮玲玉：時亥隔二位不受影響，書 −38.8。
        assert_score(&scores(["庚戌", "辛巳", "己亥", "乙亥"]), "時支", -39.8);
    }

    #[test]
    fn two_open_three_combinations_raise_water_to_sixty_percent() {
        // 量化五行力下冊 p183-184 阿儀：申子辰子兩個三合不化，二子各 1.3、申辰各 0.5，書 −7。
        let map = scores(["戊子", "丙辰", "戊子", "庚申"]);
        assert_score(&map, "年支", -35.1);
        assert_score(&map, "月支", 21.2);
        assert_score(&map, "總分", -7.3);
    }
}

#[cfg(test)]
mod tendency_tests {
    use super::tendency_label;

    #[test]
    fn tendency_follows_strength_instead_of_score_sign() {
        // 正格身強：剋洩耗（負分）才是好運。
        assert_eq!(tendency_label(-12.0, 120.0), "偏吉");
        assert_eq!(tendency_label(12.0, 120.0), "偏壓");
        // 正格身弱：扶身（正分）才是好運。
        assert_eq!(tendency_label(12.0, -80.0), "偏吉");
        assert_eq!(tendency_label(-3.0, -80.0), "需防");
        // 從強順印比之勢、從弱順食財官之勢。
        assert_eq!(tendency_label(12.0, 230.0), "偏吉");
        assert_eq!(tendency_label(-12.0, -230.0), "偏吉");
        assert_eq!(tendency_label(1.0, 50.0), "平");
    }
}
