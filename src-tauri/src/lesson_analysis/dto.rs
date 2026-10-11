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
    pub pressure_families_surface: Vec<String>,
    pub pressure_families_hidden: Vec<String>,
    pub interactions: Vec<LessonInteractionEvidence>,
    pub evidence: Vec<LessonEvidence>,
    pub limitations: Vec<String>,
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
