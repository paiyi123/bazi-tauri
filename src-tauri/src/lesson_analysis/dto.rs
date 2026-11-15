use serde::Serialize;

#[derive(Debug, Clone)]
pub(crate) struct LessonModelReference {
    pub ge_ju: String,
    pub strength_label: String,
    pub use_elements: Vec<String>,
    pub special_pattern: Option<String>,
    pub special_category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonThreeStepAnalysis {
    pub version: String,
    pub status: String,
    pub facts: LessonFacts,
    pub pattern: LessonPatternAnalysis,
    pub strength: LessonStrengthAnalysis,
    pub preferences: LessonPreferenceAnalysis,
    pub stem_interactions: LessonStemInteractionAnalysis,
    pub branch_interactions: LessonBranchInteractionAnalysis,
    pub pillar_relations: LessonPillarRelationAnalysis,
    pub interaction_strength: LessonInteractionStrengthReview,
    pub comparison: LessonComparison,
    pub sources: Vec<LessonSource>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonEvidence {
    pub id: String,
    pub kind: String,
    pub pillar: Option<String>,
    pub hidden_stem_layer: Option<String>,
    pub interaction_id: Option<String>,
    pub description: String,
    pub source_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonSource {
    pub date: String,
    pub rule_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonFacts {
    pub title: String,
    pub conclusion: String,
    pub pillars: Vec<LessonPillarFact>,
    pub day_master: LessonDayMasterFact,
    pub month_command: LessonMonthCommandFact,
    pub transparencies: Vec<LessonTransparencyFact>,
    pub roots: Vec<LessonRootFact>,
    pub ten_god_distribution: Vec<LessonFamilyDistribution>,
    pub evidence: Vec<LessonEvidence>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPillarFact {
    pub pillar: String,
    pub gan_zhi: String,
    pub stem: String,
    pub stem_element: String,
    pub stem_ten_god: String,
    pub branch: String,
    pub branch_main_element: String,
    pub hidden_stems: Vec<LessonHiddenStemFact>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonHiddenStemFact {
    pub stem: String,
    pub element: String,
    pub ten_god: String,
    pub layer: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonDayMasterFact {
    pub stem: String,
    pub element: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonMonthCommandFact {
    pub branch: String,
    pub main_stem: String,
    pub element: String,
    pub ten_god: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonTransparencyFact {
    pub hidden_stem: String,
    pub hidden_stem_layer: String,
    pub ten_god: String,
    pub visible_locations: Vec<String>,
    pub evidence_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonRootFact {
    pub pillar: String,
    pub branch: String,
    pub hidden_stem: String,
    pub hidden_stem_layer: String,
    pub element: String,
    pub ten_god: String,
    pub root_kind: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonFamilyDistribution {
    pub family: String,
    pub element: String,
    pub surface_locations: Vec<String>,
    pub hidden_locations: Vec<String>,
    pub surface_evidence_ids: Vec<String>,
    pub hidden_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPatternAnalysis {
    pub title: String,
    pub conclusion: String,
    pub status: String,
    pub ordinary_result: Option<String>,
    pub ordinary_candidates: Vec<LessonPatternCandidate>,
    pub special_review: LessonSpecialReview,
    pub evidence: Vec<LessonEvidence>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPatternCandidate {
    pub pattern: String,
    pub ten_god: String,
    pub month_hidden_stem: String,
    pub hidden_stem_layer: String,
    pub transparent_locations: Vec<String>,
    pub selection: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonSpecialReview {
    pub required: bool,
    pub reasons: Vec<String>,
    pub model_reference_pattern: Option<String>,
    pub model_reference_category: Option<String>,
    pub specialized_use_versions: Vec<LessonSpecialUseVersion>,
    pub follow_child_peer_exception: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonSpecialUseVersion {
    pub source_date: String,
    pub use_direction: String,
    pub favorable_direction: String,
    pub limitation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStrengthAnalysis {
    pub title: String,
    pub conclusion: String,
    pub status: String,
    pub classification: String,
    pub same_season_element: bool,
    pub seal_in_season: bool,
    pub support_stem_count: usize,
    pub support_branch_count: usize,
    pub same_party_count: usize,
    pub opposing_party_count: usize,
    pub month_command_same_party: bool,
    pub momentum_proxy: bool,
    pub root_count: usize,
    pub observation_count: usize,
    pub same_party_characters: Vec<String>,
    pub opposing_party_characters: Vec<String>,
    pub checks: Vec<LessonStrengthCheck>,
    pub verdict_reason: String,
    pub pressure_families_surface: Vec<String>,
    pub pressure_families_hidden: Vec<String>,
    pub interactions: Vec<LessonInteractionEvidence>,
    pub evidence: Vec<LessonEvidence>,
    pub limitations: Vec<String>,
    /// 2026-11-04：被相鄰地支刑沖打掉、不算得地的根（餘氣、中氣）。
    pub root_knockouts: Vec<String>,
    /// 母慈滅子（印太多、無比劫、無本氣根）：不論身強（講義例一、滴天髓）。
    pub mother_overload: Option<String>,
    /// 中和提示：強弱差距小、五行分佈平均（講義例三「行五運皆可，只忌歲運逢沖」）。
    pub balance_hint: Option<String>,
}

/// One of the classroom's three strength observations (得令／得勢／得地), spelled out for display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStrengthCheck {
    pub item: String,
    pub hit: bool,
    pub rule: String,
    pub finding: String,
    pub source_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonInteractionEvidence {
    pub id: String,
    pub scope: String,
    pub interaction_type: String,
    pub target: String,
    pub outcome: String,
    pub pillars: String,
    pub detail: String,
    pub assessment: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPreferenceAnalysis {
    pub title: String,
    pub conclusion: String,
    pub status: String,
    pub plans: Vec<LessonPreferencePlan>,
    pub no_plan_reasons: Vec<String>,
    pub bridge_paths: Vec<LessonBridgePath>,
    pub medicine_notes: Vec<LessonMedicineNote>,
    pub seasonal_compatibility: Vec<LessonSeasonalNote>,
    pub evidence: Vec<LessonEvidence>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPreferencePlan {
    pub rule_id: String,
    pub title: String,
    pub applicability: String,
    pub condition: String,
    pub use_directions: Vec<LessonDirection>,
    pub favorable_directions: Vec<LessonDirection>,
    pub unfavorable_directions: Vec<LessonDirection>,
    pub conditional_directions: Vec<LessonDirection>,
    pub evidence_ids: Vec<String>,
    pub rejections: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonDirection {
    pub family: String,
    pub element: String,
    pub role: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBridgePath {
    pub conflict_from: String,
    pub intermediary: String,
    pub conflict_to: String,
    pub path: Vec<String>,
    pub actual_nodes: Vec<String>,
    pub missing_nodes: Vec<String>,
    pub status: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonMedicineNote {
    pub protected_plan_rule_id: String,
    pub protected_family: String,
    pub protected_element: String,
    pub hindrance_family: String,
    pub medicine_family: Option<String>,
    pub status: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonSeasonalNote {
    pub plan_rule_id: Option<String>,
    pub season: String,
    pub month_branches: Vec<String>,
    pub status: String,
    pub element: Option<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonComparison {
    pub pattern: LessonComparisonItem,
    pub strength: LessonComparisonItem,
    pub preferences: LessonComparisonItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonComparisonItem {
    pub app_value: String,
    pub lesson_value: String,
    pub agreement_status: String,
    pub note: String,
}

/// 天干生剋合化（2026-10-07、10-14）：五合的五種變化與天干生剋的遠近、通關。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStemInteractionAnalysis {
    pub title: String,
    pub conclusion: String,
    pub status: String,
    pub combinations: Vec<LessonStemCombination>,
    pub relations: Vec<LessonStemRelation>,
    pub limitations: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStemCombination {
    pub id: String,
    pub stems: String,
    pub positions: Vec<String>,
    pub element: String,
    /// 日干本身之合／日干以外之合／隔位相合
    pub scope: String,
    /// blocked／bound／contested／transformed
    pub kind: String,
    /// 合而不合／合而不化／爭合妒合／合而能化
    pub kind_label: String,
    /// 合來／合去
    pub direction: Option<String>,
    pub bound_stems: Vec<String>,
    /// good／bad／mixed／neutral
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    pub notes: Vec<String>,
    pub needs_confirmation: bool,
    pub source_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonStemRelation {
    pub id: String,
    pub from: String,
    pub to: String,
    /// 生／剋
    pub relation: String,
    /// 鄰近／隔一位／遙隔
    pub distance: String,
    /// active／weak／ignored
    pub status: String,
    pub status_label: String,
    pub reason: String,
    pub source_date: String,
}

/// 地支沖合（2026-10-21）：六沖的勝負與去喜去忌、六合的生合剋合與解沖。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBranchInteractionAnalysis {
    pub title: String,
    pub conclusion: String,
    pub status: String,
    pub clashes: Vec<LessonBranchClash>,
    pub combinations: Vec<LessonBranchCombination>,
    /// 三合、半合、三會、辰戌丑未全（講義第二十四章第三、四節）
    pub gatherings: Vec<LessonBranchGathering>,
    /// 三刑、自刑、六害，以本氣生剋論（講義第二十四章第五節）
    pub punishments: Vec<LessonBranchPunishment>,
    pub limitations: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBranchClash {
    pub id: String,
    pub branches: String,
    pub positions: Vec<String>,
    /// 貼近／隔一位／遙隔
    pub distance: String,
    pub force_label: String,
    pub winner: Option<String>,
    pub loser: Option<String>,
    /// decisive／both_damaged／storage
    pub outcome: String,
    pub outcome_label: String,
    /// good／bad／mixed／neutral
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    pub notes: Vec<String>,
    pub source_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBranchCombination {
    pub id: String,
    pub branches: String,
    pub positions: Vec<String>,
    /// generating（生合）／controlling（剋合）
    pub kind: String,
    pub kind_label: String,
    pub adjacent: bool,
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    /// 與相沖並見時的作用（加重沖力、減低沖力）
    pub clash_effect: Option<String>,
    pub notes: Vec<String>,
    pub source_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBranchGathering {
    pub id: String,
    /// three_combination／three_meeting／half_combination／incomplete_half／earth_full
    pub kind: String,
    pub kind_label: String,
    pub branches: String,
    pub positions: Vec<String>,
    /// 局的五行
    pub element: String,
    /// 是否成局（不成局時依常理論）
    pub formed: bool,
    pub force_label: String,
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    pub notes: Vec<String>,
    pub source_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonBranchPunishment {
    pub id: String,
    /// punishment／self_punishment／harm
    pub kind: String,
    pub kind_label: String,
    pub branches: String,
    pub positions: Vec<String>,
    /// 本氣生剋讀法，例如「木生火、火剋金」
    pub relation: String,
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    pub notes: Vec<String>,
    pub source_date: String,
}

/// 同柱干支生剋合（講義第二十五章）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPillarRelationAnalysis {
    pub title: String,
    pub conclusion: String,
    pub relations: Vec<LessonPillarRelation>,
    pub limitations: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPillarRelation {
    pub id: String,
    pub pillar: String,
    pub gan_zhi: String,
    /// same／stem_generates_branch／branch_generates_stem／stem_controls_branch／branch_controls_stem
    pub kind: String,
    pub kind_label: String,
    pub hidden_combination: Option<String>,
    pub hidden_combination_pure: bool,
    pub stem_favor: String,
    pub verdict: String,
    pub verdict_label: String,
    pub reason: String,
    pub notes: Vec<String>,
    pub source_date: String,
}

/// 作用變化後強弱複核（講義第二十六章）：只做提醒，不改課堂判定。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonInteractionStrengthReview {
    pub title: String,
    pub conclusion: String,
    pub same_party_count: usize,
    pub opposing_party_count: usize,
    pub momentum: bool,
    pub classification: String,
    pub changed: bool,
    pub changes: Vec<String>,
    pub limitations: Vec<String>,
    pub sources: Vec<String>,
}
