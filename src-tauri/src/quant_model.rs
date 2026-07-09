use crate::bazi::{
    DaYunDto, LiuNianDto, QuantAnnualLuckScore, QuantCongPattern, QuantLuckScore,
    QuantModelInteraction, QuantModelPillarScore, QuantModelResponse, QuantModelSubScore,
    QuantShaYin, QuantYongShen,
};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use tyme4rs::tyme::sixtycycle::{EarthBranch, HeavenStem, SixtyCycle};
use tyme4rs::tyme::Culture;

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
            RootLevel::Middle | RootLevel::Tomb => 2,
            RootLevel::Lu => 3,
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
    let sha_yin = Some(analyze_sha_yin(
        &day_master_name,
        total,
        &bundle.stem_scores,
        &bundle.branch_scores,
    ));
    let interactions = Some(build_natal_interactions(
        &bundle.resolved_state,
        &bundle.branch_scores,
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
        sha_yin,
        interactions,
        stem_scores: bundle.stem_scores,
        branch_scores: bundle.branch_scores,
        luck_scores,
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
        resolve_three_meeting_transforms(&three_meeting_candidates, &inputs.branches);
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

    let six_combination_effects = resolve_six_combination_effects(
        &inputs.branches,
        &half_combination_candidates,
        &three_combination_candidates,
        &three_meeting_candidates,
        &branch_transforms,
    );
    let branch_prune_effects = resolve_branch_prune_effects(&inputs.branches);
    let branch_qi_modifiers = resolve_branch_qi_modifiers(
        &inputs.branches,
        &six_combination_effects,
        &three_meeting_candidates,
    );
    let stem_combination_effects = resolve_stem_combination_effects(
        &inputs.stems,
        &inputs.branches[1],
        branch_transforms.get(&1),
    );
    let stem_clash_effects = resolve_stem_clash_effects(&inputs.stems);

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
                    distance(input.position_index, branch.position_index),
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

    let slices = if let Some(transform) = branch_transform {
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

    if branch_transform.is_none() && is_element_lu_branch(day_master, &input.branch) {
        let lu_element = stem_element(day_master);
        let lu_stem = stem_for_element(&lu_element, is_yang(day_master));
        adjusted_raw_total = 100.0;
        let qi_multiplier =
            branch_score_qi_multiplier(input, &lu_stem, &branch_interaction, branch_qi_modifier);
        position_adjusted_total = round1(weight * branch_interaction.multiplier * qi_multiplier);
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

fn build_luck_scores(
    day_master: &str,
    natal_total: f64,
    natal_inputs: &CalculationInputs,
    da_yun: &[DaYunDto],
) -> Vec<QuantLuckScore> {
    da_yun
        .iter()
        .filter_map(|item| {
            let (luck_stem, luck_branch) = split_gan_zhi(&item.gan_zhi)?;
            let extended_inputs = build_extended_inputs(
                natal_inputs,
                Some(&luck_stem),
                Some(&luck_branch),
                None,
                None,
            );
            let bundle = score_resolved_chart(day_master, &extended_inputs);
            let luck_stem_pillar = resolve_pillar_score(&bundle.stem_scores, "運干");
            let luck_branch_pillar = resolve_pillar_score(&bundle.branch_scores, "運支");
            let stem_score = luck_stem_pillar.map(|item| item.final_score).unwrap_or(0.0);
            let branch_score = luck_branch_pillar
                .map(|item| item.final_score)
                .unwrap_or(0.0);
            let total_score = round1(stem_score + branch_score);
            let impact_ratio = ratio(total_score, natal_total);
            let effective_natal_score = round1(natal_total + total_score);
            let annual_scores = if item.liu_nian.is_empty() {
                None
            } else {
                Some(build_annual_scores(
                    day_master,
                    natal_total,
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
                tendency: tendency_label(total_score),
                overview_lines: Some(vec![
                    format!("前五年看運干：{:.1}", stem_score),
                    format!("後五年看運支：{:.1}", branch_score),
                    format!("整柱參考：{:.1}", total_score),
                    format!("R值：{:.2}", impact_ratio),
                    format!("作用後命局：{:.1}", effective_natal_score),
                ]),
                interaction_lines: Some(build_pillar_detail_lines(
                    luck_stem_pillar,
                    luck_branch_pillar,
                )),
                scoring_lines: Some(vec![
                    format!("運干列分：{:.1}", stem_score),
                    format!("運支列分：{:.1}", branch_score),
                ]),
                details: format!(
                    "原局 + 大運同場裁決後，抽出運干 {} 與運支 {} 的列分。",
                    luck_stem, luck_branch
                ),
                annual_scores,
            })
        })
        .collect()
}

fn build_annual_scores(
    day_master: &str,
    natal_total: f64,
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
                natal_inputs,
                &luck_stem,
                &luck_branch,
                item,
            )
        })
        .collect()
}

fn build_annual_score(
    day_master: &str,
    natal_total: f64,
    natal_inputs: &CalculationInputs,
    luck_stem: &str,
    luck_branch: &str,
    item: &LiuNianDto,
) -> Option<QuantAnnualLuckScore> {
    let (annual_stem, annual_branch) = split_gan_zhi(&item.gan_zhi)?;
    let extended_inputs = build_extended_inputs(
        natal_inputs,
        Some(luck_stem),
        Some(luck_branch),
        Some(&annual_stem),
        Some(&annual_branch),
    );
    let bundle = score_resolved_chart(day_master, &extended_inputs);
    let annual_stem_pillar = resolve_pillar_score(&bundle.stem_scores, "流年干");
    let annual_branch_pillar = resolve_pillar_score(&bundle.branch_scores, "流年支");
    let luck_stem_pillar = resolve_pillar_score(&bundle.stem_scores, "運干");
    let luck_branch_pillar = resolve_pillar_score(&bundle.branch_scores, "運支");
    let annual_stem_score = annual_stem_pillar
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let annual_branch_score = annual_branch_pillar
        .map(|item| item.final_score)
        .unwrap_or(0.0);
    let annual_total_score = round1(annual_stem_score + annual_branch_score);
    let full_luck_score = round1(
        luck_stem_pillar.map(|item| item.final_score).unwrap_or(0.0)
            + luck_branch_pillar
                .map(|item| item.final_score)
                .unwrap_or(0.0),
    );
    let first_half = item.index < 5;
    let active_luck_label = if first_half {
        format!("大運前五年：運干{}", luck_stem)
    } else {
        format!("大運後五年：運支{}", luck_branch)
    };
    let active_luck_score = if first_half {
        luck_stem_pillar.map(|item| item.final_score).unwrap_or(0.0)
    } else {
        luck_branch_pillar
            .map(|item| item.final_score)
            .unwrap_or(0.0)
    };
    let combined_score = round1(active_luck_score + annual_total_score);
    let full_combined_score = round1(full_luck_score + annual_total_score);
    let impact_ratio = ratio(combined_score, natal_total);
    let full_impact_ratio = ratio(full_combined_score, natal_total);
    let effective_natal_score = round1(natal_total + combined_score);
    let full_effective_natal_score = round1(natal_total + full_combined_score);

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
        tendency: tendency_label(combined_score),
        full_tendency: tendency_label(full_combined_score),
        overview_lines: Some(vec![
            format!("流年干：{:.1}", annual_stem_score),
            format!("流年支：{:.1}", annual_branch_score),
            format!("流年整柱：{:.1}", annual_total_score),
            format!("半運合參：{:.1}", combined_score),
            format!("整運合參：{:.1}", full_combined_score),
        ]),
        interaction_lines: Some(build_pillar_detail_lines(
            annual_stem_pillar,
            annual_branch_pillar,
        )),
        scoring_lines: Some(vec![
            format!("R值（半運）：{:.2}", impact_ratio),
            format!("R值（整運）：{:.2}", full_impact_ratio),
            format!("作用後命局（半運）：{:.1}", effective_natal_score),
            format!("作用後命局（整運）：{:.1}", full_effective_natal_score),
        ]),
        details: format!(
            "原局 + 大運 + 流年同場裁決後，抽出流年干 {} 與流年支 {} 的列分。",
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
            position_index: stems.len(),
        });
    }
    if let Some(stem) = annual_stem {
        stems.push(StemInput {
            pillar: "流年干".to_string(),
            stem: stem.to_string(),
            position_index: stems.len(),
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
    add_stem_clash_rows(&mut rows, &state.stems);
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

fn add_stem_clash_rows(rows: &mut Vec<QuantModelInteraction>, stems: &[StemInput]) {
    for i in 0..stems.len() {
        for j in (i + 1)..stems.len() {
            let left = &stems[i];
            let right = &stems[j];
            if !is_stem_clash(&left.stem, &right.stem) {
                continue;
            }
            let dist = distance(left.position_index, right.position_index);
            if !is_supported_stem_clash_distance(left, right, dist) {
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
        ("寅", "巳", "刑", "寅巳申三刑(無恩之刑)"),
        ("巳", "申", "刑", "寅巳申三刑(無恩之刑)"),
        ("寅", "申", "刑", "寅巳申三刑(無恩之刑)"),
        ("丑", "戌", "刑", "丑戌未三刑(恃勢之刑)"),
        ("戌", "未", "刑", "丑戌未三刑(恃勢之刑)"),
        ("丑", "未", "刑", "丑戌未三刑(恃勢之刑)"),
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
    let month_branch = &branches[1];
    let mut candidates = Vec::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            for k in (j + 1)..branches.len() {
                if let Some((element, label)) = resolve_three_meeting(
                    &branches[i].branch,
                    &branches[j].branch,
                    &branches[k].branch,
                ) {
                    let transforms = should_transform_branch_combination(
                        &element,
                        month_branch,
                        stems,
                        day_master,
                    );
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
    let month_branch = &branches[1];
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
                        month_branch,
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
    let month_branch = &branches[1];
    let mut candidates = Vec::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            if distance(branches[i].position_index, branches[j].position_index) > 2 {
                continue;
            }
            if let Some((element, note, combine_note)) =
                resolve_half_combination(&branches[i].branch, &branches[j].branch)
            {
                let transforms =
                    should_transform_branch_combination(&element, month_branch, stems, day_master);
                candidates.push(HalfCombinationCandidate {
                    left_index: branches[i].position_index,
                    right_index: branches[j].position_index,
                    element,
                    distance: distance(branches[i].position_index, branches[j].position_index),
                    transforms,
                    note,
                    combine_note,
                });
            }
        }
    }
    candidates
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
) -> HashMap<usize, BranchTransform> {
    let mut transforms = HashMap::new();
    for candidate in candidates.iter().filter(|item| item.transforms) {
        if is_suppressed_by_direct_clashed_specialist(candidate, branches)
            || is_suppressed_by_stacked_six_combination(candidate, branches)
            || three_combination_candidates.iter().any(|item| {
                item.involves(candidate.left_index) && item.involves(candidate.right_index)
            })
            || three_meeting_candidates.iter().any(|item| {
                item.involves(candidate.left_index) && item.involves(candidate.right_index)
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

fn resolve_six_combination_transforms(
    branches: &[BranchInput],
    stems: &[StemInput],
    day_master: &str,
    half_combination_candidates: &[HalfCombinationCandidate],
    three_combination_candidates: &[ThreeCombinationCandidate],
    three_meeting_candidates: &[ThreeMeetingCandidate],
    existing_transforms: &HashMap<usize, BranchTransform>,
) -> HashMap<usize, BranchTransform> {
    let month_branch = &branches[1];
    let mut transforms = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            if distance(left.position_index, right.position_index) != 1
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
                || !should_transform_branch_combination(&element, month_branch, stems, day_master)
            {
                continue;
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
            if distance(left.position_index, right.position_index) != 1
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
            effects.insert(
                left.position_index,
                SixCombinationEffect {
                    other_index: right.position_index,
                    pair: pair.clone(),
                    element: element.clone(),
                    remainder: 0.52,
                },
            );
            effects.insert(
                right.position_index,
                SixCombinationEffect {
                    other_index: left.position_index,
                    pair,
                    element,
                    remainder: 0.52,
                },
            );
        }
    }
    effects
}

fn resolve_branch_prune_effects(branches: &[BranchInput]) -> HashMap<usize, BranchPruneEffect> {
    let mut effects = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            if is_chen_xu_pair(&left.branch, &right.branch)
                && distance(left.position_index, right.position_index) == 1
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

fn resolve_branch_qi_modifiers(
    branches: &[BranchInput],
    six_combination_effects: &HashMap<usize, SixCombinationEffect>,
    three_meeting_candidates: &[ThreeMeetingCandidate],
) -> HashMap<usize, BranchQiModifier> {
    let mut modifiers = HashMap::new();
    for i in 0..branches.len() {
        for j in (i + 1)..branches.len() {
            let left = &branches[i];
            let right = &branches[j];
            let dist = distance(left.position_index, right.position_index);
            if !is_direct_clash(&left.branch, &right.branch)
                || dist > 2
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
            let affected_multiplier = branch_clash_affected_multiplier(dist);
            merge_branch_qi_modifier_into(
                &mut modifiers,
                left.position_index,
                build_branch_clash_qi_modifier(
                    &left.branch,
                    &right.branch,
                    clash_type,
                    affected_multiplier,
                    dist,
                ),
            );
            merge_branch_qi_modifier_into(
                &mut modifiers,
                right.position_index,
                build_branch_clash_qi_modifier(
                    &right.branch,
                    &left.branch,
                    clash_type,
                    affected_multiplier,
                    dist,
                ),
            );
        }
    }
    for i in 0..branches.len().saturating_sub(2) {
        let left = &branches[i];
        let middle = &branches[i + 1];
        let right = &branches[i + 2];
        if left.branch == "寅" && middle.branch == "申" && right.branch == "寅" {
            let mut stem_multipliers = HashMap::new();
            stem_multipliers.insert("甲".to_string(), 0.5);
            stem_multipliers.insert("丙".to_string(), 0.5);
            let modifier = BranchQiModifier {
                stem_multipliers,
                element_multipliers: HashMap::new(),
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
            if distance(left.position_index, right.position_index) > 1 {
                continue;
            }
            let Some((pair, target_element)) = resolve_stem_combination(&left.stem, &right.stem)
            else {
                continue;
            };
            let month_element =
                resolve_stem_combination_month_element(month_branch, month_transform);
            let transforms = month_element == target_element;
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

fn resolve_stem_clash_effects(stems: &[StemInput]) -> HashMap<usize, StemClashEffect> {
    let mut effects = HashMap::new();
    for i in 0..stems.len() {
        for j in (i + 1)..stems.len() {
            let left = &stems[i];
            let right = &stems[j];
            if !is_stem_clash(&left.stem, &right.stem) {
                continue;
            }
            let dist = distance(left.position_index, right.position_index);
            if !is_supported_stem_clash_distance(left, right, dist) {
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
        let visible_element = stem_element(&stem.stem);
        if visible_element != hidden_element {
            continue;
        }
        let dist = distance(branch.position_index, stem.position_index);
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
        if stem_element(&stem.stem) != transformed_element {
            continue;
        }
        let dist = distance(branch.position_index, stem.position_index);
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
        let dist = distance(input.position_index, other.position_index);
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
            if has_stronger_three_combination(input, other, &state.three_combination_candidates)
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
            let candidate = if dist <= 1 {
                BranchInteraction::combine("近合".to_string(), 0.85, other)
            } else if dist == 2 {
                BranchInteraction::combine("隔柱合".to_string(), 0.9, other)
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
    month_branch: &BranchInput,
    stems: &[StemInput],
    day_master: &str,
) -> bool {
    if !has_visible_transforming_stem(target_element, stems, day_master) {
        return false;
    }
    let month_main_stem = resolve_month_main_stem(month_branch);
    let month_element = stem_element(&month_main_stem);
    !controls(&month_element, target_element)
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

fn is_suppressed_by_direct_clashed_specialist(
    candidate: &HalfCombinationCandidate,
    branches: &[BranchInput],
) -> bool {
    let left = &branches[candidate.left_index];
    let right = &branches[candidate.right_index];
    specialist_branch_blocked_by_direct_clash(left, &candidate.element, branches)
        || specialist_branch_blocked_by_direct_clash(right, &candidate.element, branches)
}

fn specialist_branch_blocked_by_direct_clash(
    branch: &BranchInput,
    target_element: &str,
    branches: &[BranchInput],
) -> bool {
    if !is_half_combination_specialist(&branch.branch, target_element) {
        return false;
    }
    branches.iter().any(|other| {
        other.position_index != branch.position_index
            && distance(branch.position_index, other.position_index) == 1
            && is_direct_clash(&branch.branch, &other.branch)
            && classify_branch_clash(&branch.branch, &other.branch) == BranchClashType::Specialist
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

fn has_stronger_three_meeting(
    input: &BranchInput,
    other: &BranchInput,
    candidates: &[ThreeMeetingCandidate],
) -> bool {
    candidates.iter().any(|candidate| {
        (candidate.involves(input.position_index) || candidate.involves(other.position_index))
            && !(candidate.involves(input.position_index)
                && candidate.involves(other.position_index))
    })
}

fn should_suppress_clash_by_six_combination(
    input: &BranchInput,
    other: &BranchInput,
    effects: &HashMap<usize, SixCombinationEffect>,
) -> bool {
    distance(input.position_index, other.position_index) > 1
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

fn build_branch_clash_qi_modifier(
    branch: &str,
    other_branch: &str,
    clash_type: BranchClashType,
    affected_multiplier: f64,
    dist: usize,
) -> BranchQiModifier {
    let stem_multipliers = match clash_type {
        BranchClashType::Specialist => hidden_stems_of(branch)
            .into_iter()
            .map(|stem| (stem, affected_multiplier))
            .collect(),
        BranchClashType::Growth => match branch {
            "寅" => vec![
                ("甲".to_string(), affected_multiplier),
                ("丙".to_string(), affected_multiplier),
            ],
            "申" => vec![
                ("庚".to_string(), affected_multiplier),
                ("壬".to_string(), affected_multiplier),
            ],
            "巳" => vec![
                ("丙".to_string(), affected_multiplier),
                ("庚".to_string(), affected_multiplier),
            ],
            "亥" => vec![
                ("壬".to_string(), affected_multiplier),
                ("甲".to_string(), affected_multiplier),
            ],
            _ => Vec::new(),
        }
        .into_iter()
        .collect(),
        BranchClashType::Storage => match branch {
            "辰" => vec![
                ("乙".to_string(), affected_multiplier),
                ("癸".to_string(), affected_multiplier),
            ],
            "戌" => vec![
                ("辛".to_string(), affected_multiplier),
                ("丁".to_string(), affected_multiplier),
            ],
            "丑" => vec![
                ("癸".to_string(), affected_multiplier),
                ("辛".to_string(), affected_multiplier),
            ],
            "未" => vec![
                ("丁".to_string(), affected_multiplier),
                ("乙".to_string(), affected_multiplier),
            ],
            _ => Vec::new(),
        }
        .into_iter()
        .collect(),
    };
    let element_multipliers = if clash_type == BranchClashType::Specialist {
        let mut map = HashMap::new();
        map.insert(resolve_branch_element(branch), affected_multiplier);
        for hidden in hidden_stems_of(branch) {
            map.insert(stem_element(&hidden), affected_multiplier);
        }
        map
    } else {
        HashMap::new()
    };
    let note = match clash_type {
        BranchClashType::Specialist => {
            clash_type_distance_note("專氣相沖", dist, affected_multiplier)
        }
        BranchClashType::Growth => clash_type_distance_note("長生相沖", dist, affected_multiplier),
        BranchClashType::Storage => clash_type_distance_note("墓庫相沖", dist, affected_multiplier),
    };
    let _ = other_branch;
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
        "申戌酉" => Some(("金".to_string(), "申酉戌".to_string())),
        "亥子丑" => Some(("水".to_string(), "亥子丑".to_string())),
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
        "運干" => 35.0,
        "流年干" => 20.0,
        "年支" => 45.0,
        "月支" => 100.0,
        "日支" => 55.0,
        "時支" => 50.0,
        "運支" => 25.0,
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

fn resolve_three_combination_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    if role == CombinationBranchRole::Specialist {
        1.4
    } else {
        1.0
    }
}

fn resolve_three_combination_residual_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    if role == CombinationBranchRole::Specialist {
        0.0
    } else {
        0.08
    }
}

fn resolve_three_meeting_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    match role {
        CombinationBranchRole::Specialist | CombinationBranchRole::Growth => 2.1,
        CombinationBranchRole::Storage => 1.0,
        CombinationBranchRole::Other => 1.0,
    }
}

fn resolve_three_meeting_residual_factor(_label: &str, role: CombinationBranchRole) -> f64 {
    if role == CombinationBranchRole::Storage {
        0.2
    } else {
        0.0
    }
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

fn is_supported_stem_clash_distance(left: &StemInput, right: &StemInput, dist: usize) -> bool {
    dist == 1
        || (dist == 2
            && left.pillar == "月干"
            && right.pillar == "時干"
            && sorted_pair(&left.stem, &right.stem) == "庚甲")
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

fn branch_clash_score_multiplier(clash_type: BranchClashType, dist: usize) -> f64 {
    if clash_type == BranchClashType::Specialist {
        match dist {
            1 => 0.0,
            2 => 0.75,
            _ => 0.89,
        }
    } else {
        1.0
    }
}

fn branch_clash_affected_multiplier(dist: usize) -> f64 {
    match dist {
        1 => 0.0,
        2 => 0.75,
        _ => 0.89,
    }
}

fn branch_clash_note(clash_type: BranchClashType, dist: usize) -> String {
    match clash_type {
        BranchClashType::Specialist => match dist {
            1 => "近沖".to_string(),
            2 => "隔一位沖".to_string(),
            _ => "隔二位沖".to_string(),
        },
        BranchClashType::Growth => match dist {
            1 => "近沖（長生受沖支氣化零）".to_string(),
            2 => "隔一位沖（長生受沖支氣×0.75）".to_string(),
            _ => "隔二位沖（長生受沖支氣×0.89）".to_string(),
        },
        BranchClashType::Storage => match dist {
            1 => "近沖（墓庫旁氣化零）".to_string(),
            2 => "隔一位沖（墓庫旁氣×0.75）".to_string(),
            _ => "隔二位沖（墓庫旁氣×0.89）".to_string(),
        },
    }
}

fn clash_type_distance_note(prefix: &str, dist: usize, affected_multiplier: f64) -> String {
    match dist {
        1 => format!("{}：受沖支氣化零，未受沖主氣不變", prefix),
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
        RootLevel::Middle | RootLevel::Tomb => match normalize_distance(dist) {
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

fn resolve_stem_combination_month_element(
    month_branch: &BranchInput,
    month_transform: Option<&BranchTransform>,
) -> String {
    if month_transform.is_some_and(|transform| transform.combine_note.contains("@half")) {
        month_transform.unwrap().element.clone()
    } else {
        stem_element(&resolve_month_main_stem(month_branch))
    }
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
    if index == 0 || index >= branches.len() - 1 {
        return false;
    }
    let left = &branches[index - 1];
    let right = &branches[index + 1];
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

fn tendency_label(score: f64) -> String {
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

fn ratio(delta: f64, natal_total: f64) -> f64 {
    let base = natal_total.abs();
    if base < 0.1 {
        0.0
    } else {
        round2(delta / base)
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
