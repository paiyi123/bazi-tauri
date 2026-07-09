use crate::bazi::BaziResponse;
use serde::Serialize;

const PILLAR_ORDER: [&str; 4] = ["year", "month", "day", "hour"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShenShaMatchDto {
    pub name: String,
    pub basis: String,
    pub matched_pillars: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FourPillarShenShaDto {
    pub note: String,
    pub year: Vec<String>,
    pub month: Vec<String>,
    pub day: Vec<String>,
    pub hour: Vec<String>,
    pub matches: Vec<ShenShaMatchDto>,
}

#[derive(Debug, Clone)]
struct MutableMatch {
    name: String,
    bases: Vec<String>,
    pillars: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PillarMatches {
    year: Vec<String>,
    month: Vec<String>,
    day: Vec<String>,
    hour: Vec<String>,
}

pub fn analyze_shen_sha(response: &BaziResponse) -> FourPillarShenShaDto {
    if response.year_pillar.gan_zhi.is_empty()
        || response.month_pillar.gan_zhi.is_empty()
        || response.day_pillar.gan_zhi.is_empty()
        || response.hour_pillar.gan_zhi.is_empty()
    {
        return FourPillarShenShaDto {
            note: "神煞資料不足，無法判定。".to_string(),
            year: vec![],
            month: vec![],
            day: vec![],
            hour: vec![],
            matches: vec![],
        };
    }

    let stems = [
        ("year", response.year_pillar.stem.as_str()),
        ("month", response.month_pillar.stem.as_str()),
        ("day", response.day_pillar.stem.as_str()),
        ("hour", response.hour_pillar.stem.as_str()),
    ];
    let branches = [
        ("year", response.year_pillar.branch.as_str()),
        ("month", response.month_pillar.branch.as_str()),
        ("day", response.day_pillar.branch.as_str()),
        ("hour", response.hour_pillar.branch.as_str()),
    ];

    let mut pillar_matches = PillarMatches::default();
    let mut aggregated_matches: Vec<MutableMatch> = Vec::new();

    add_stem_rule(
        "天乙貴人",
        "年干",
        get_stem(&stems, "year"),
        tian_yi_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "天乙貴人",
        "日干",
        get_stem(&stems, "day"),
        tian_yi_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "文昌貴人",
        "年干",
        get_stem(&stems, "year"),
        wen_chang_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "文昌貴人",
        "日干",
        get_stem(&stems, "day"),
        wen_chang_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "太極貴人",
        "年干",
        get_stem(&stems, "year"),
        tai_ji_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "太極貴人",
        "日干",
        get_stem(&stems, "day"),
        tai_ji_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "金輿",
        "年干",
        get_stem(&stems, "year"),
        jin_yu_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "金輿",
        "日干",
        get_stem(&stems, "day"),
        jin_yu_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "羊刃",
        "日干",
        get_stem(&stems, "day"),
        yang_ren_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "國印貴人",
        "年干",
        get_stem(&stems, "year"),
        guo_yin_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "國印貴人",
        "日干",
        get_stem(&stems, "day"),
        guo_yin_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "福星貴人",
        "年干",
        get_stem(&stems, "year"),
        fu_xing_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "福星貴人",
        "日干",
        get_stem(&stems, "day"),
        fu_xing_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "天廚貴人",
        "年干",
        get_stem(&stems, "year"),
        tian_chu_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "天廚貴人",
        "日干",
        get_stem(&stems, "day"),
        tian_chu_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_rule(
        "飛刃",
        "日干",
        get_stem(&stems, "day"),
        fei_ren_by_stem,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );

    add_mixed_month_rule(
        "天德貴人",
        "月支",
        get_branch(&branches, "month"),
        tian_de_by_month_branch,
        &stems,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_mixed_month_rule(
        "月德貴人",
        "月支",
        get_branch(&branches, "month"),
        yue_de_by_month_branch,
        &stems,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_stem_target_rule(
        "月德合",
        "月支",
        get_branch(&branches, "month"),
        yue_de_by_month_branch,
        &stems,
        &mut aggregated_matches,
        &mut pillar_matches,
        true,
    );
    add_stem_list_rule(
        "德秀貴人",
        "月支",
        get_branch(&branches, "month"),
        de_xiu_by_month_branch,
        &stems,
        &mut aggregated_matches,
        &mut pillar_matches,
    );

    add_branch_rule(
        "桃花",
        "年支",
        get_branch(&branches, "year"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("酉", "卯", "子", "午"),
    );
    add_branch_rule(
        "桃花",
        "日支",
        get_branch(&branches, "day"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("酉", "卯", "子", "午"),
    );
    add_branch_rule(
        "驛馬",
        "年支",
        get_branch(&branches, "year"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("寅", "申", "巳", "亥"),
    );
    add_branch_rule(
        "驛馬",
        "日支",
        get_branch(&branches, "day"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("寅", "申", "巳", "亥"),
    );
    add_branch_rule(
        "華蓋",
        "年支",
        get_branch(&branches, "year"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("辰", "戌", "未", "丑"),
    );
    add_branch_rule(
        "華蓋",
        "日支",
        get_branch(&branches, "day"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("辰", "戌", "未", "丑"),
    );
    add_branch_rule(
        "將星",
        "年支",
        get_branch(&branches, "year"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("子", "午", "卯", "酉"),
    );
    add_branch_rule(
        "將星",
        "日支",
        get_branch(&branches, "day"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
        ("子", "午", "卯", "酉"),
    );
    add_direct_branch_rule(
        "紅鸞",
        "年支",
        get_branch(&branches, "year"),
        hong_luan_by_year_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "天喜",
        "年支",
        get_branch(&branches, "year"),
        tian_xi_by_year_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "孤辰",
        "年支",
        get_branch(&branches, "year"),
        gu_chen_by_year_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "寡宿",
        "年支",
        get_branch(&branches, "year"),
        gua_su_by_year_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "劫煞",
        "年支",
        get_branch(&branches, "year"),
        jie_sha_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "劫煞",
        "日支",
        get_branch(&branches, "day"),
        jie_sha_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "災煞",
        "年支",
        get_branch(&branches, "year"),
        zai_sha_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "災煞",
        "日支",
        get_branch(&branches, "day"),
        zai_sha_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "亡神",
        "年支",
        get_branch(&branches, "year"),
        wang_shen_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "亡神",
        "日支",
        get_branch(&branches, "day"),
        wang_shen_by_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_direct_branch_rule(
        "喪門",
        "年支",
        get_branch(&branches, "year"),
        sang_men_by_year_branch,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_special_branch_rule(
        "天羅地網",
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_kong_wang_rule(
        response,
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_tong_zi_rule(
        get_branch(&branches, "month"),
        &branches,
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_kui_gang_rule(response, &mut aggregated_matches, &mut pillar_matches);
    add_fixed_day_pillar_rule(
        "十惡大敗",
        response,
        &[
            "甲辰", "乙巳", "丙申", "丁亥", "戊戌", "己丑", "庚辰", "辛巳", "壬申", "癸亥",
        ],
        &mut aggregated_matches,
        &mut pillar_matches,
    );
    add_fixed_day_pillar_rule(
        "陰差陽錯",
        response,
        &[
            "丙子", "丁丑", "戊寅", "辛卯", "壬辰", "癸巳", "丙午", "丁未", "戊申", "辛酉", "壬戌",
            "癸亥",
        ],
        &mut aggregated_matches,
        &mut pillar_matches,
    );

    FourPillarShenShaDto {
        note: "目前列 31 項常用神煞：天乙貴人、文昌貴人、太極貴人、天德貴人、月德貴人、月德合、德秀貴人、桃花、驛馬、華蓋、將星、紅鸞、天喜、孤辰、寡宿、金輿、羊刃、飛刃、劫煞、災煞、亡神、國印貴人、魁罡、福星貴人、十惡大敗、陰差陽錯、天廚貴人、天羅地網、空亡、童子煞、喪門；規則以年干/日干、月支、年支/日支、旬空與固定日柱查四柱。".to_string(),
        year: pillar_matches.year,
        month: pillar_matches.month,
        day: pillar_matches.day,
        hour: pillar_matches.hour,
        matches: aggregated_matches
            .into_iter()
            .map(|item| ShenShaMatchDto {
                name: item.name,
                basis: item.bases.join(" / "),
                matched_pillars: item
                    .pillars
                    .into_iter()
                    .map(|pillar| pillar_label(&pillar))
                    .map(ToString::to_string)
                    .collect(),
            })
            .collect(),
    }
}

fn get_stem<'a>(stems: &'a [(&str, &str)], key: &str) -> &'a str {
    stems
        .iter()
        .find(|(pillar, _)| *pillar == key)
        .map(|(_, stem)| *stem)
        .unwrap_or("")
}

fn get_branch<'a>(branches: &'a [(&str, &str)], key: &str) -> &'a str {
    branches
        .iter()
        .find(|(pillar, _)| *pillar == key)
        .map(|(_, branch)| *branch)
        .unwrap_or("")
}

fn pillar_label(key: &str) -> &'static str {
    match key {
        "year" => "年柱",
        "month" => "月柱",
        "day" => "日柱",
        "hour" => "時柱",
        _ => "",
    }
}

fn record_match(
    name: &str,
    basis: String,
    matched_pillars: Vec<&str>,
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let entry = if let Some(existing) = aggregated_matches.iter_mut().find(|item| item.name == name)
    {
        existing
    } else {
        aggregated_matches.push(MutableMatch {
            name: name.to_string(),
            bases: vec![],
            pillars: vec![],
        });
        aggregated_matches.last_mut().unwrap()
    };

    push_unique(&mut entry.bases, basis);
    for pillar in matched_pillars {
        push_unique(&mut entry.pillars, pillar.to_string());
        match pillar {
            "year" => push_unique(&mut pillar_matches.year, name.to_string()),
            "month" => push_unique(&mut pillar_matches.month, name.to_string()),
            "day" => push_unique(&mut pillar_matches.day, name.to_string()),
            "hour" => push_unique(&mut pillar_matches.hour, name.to_string()),
            _ => {}
        }
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn add_stem_rule(
    name: &str,
    basis_prefix: &str,
    reference_stem: &str,
    rule_table: fn(&str) -> &'static [&'static str],
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let target_branches = rule_table(reference_stem);
    if target_branches.is_empty() {
        return;
    }
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| target_branches.contains(&get_branch(branches, pillar)))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_stem}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_mixed_month_rule(
    name: &str,
    basis_prefix: &str,
    reference_branch: &str,
    rule_table: fn(&str) -> Option<&'static str>,
    stems: &[(&str, &str)],
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let Some(target) = rule_table(reference_branch) else {
        return;
    };
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| {
            matches_mixed_target(
                target,
                get_stem(stems, pillar),
                get_branch(branches, pillar),
            )
        })
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_stem_target_rule(
    name: &str,
    basis_prefix: &str,
    reference_branch: &str,
    rule_table: fn(&str) -> Option<&'static str>,
    stems: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
    use_combination_partner: bool,
) {
    let Some(target) = rule_table(reference_branch) else {
        return;
    };
    let effective_target = if use_combination_partner {
        five_combination_partner(target).unwrap_or(target)
    } else {
        target
    };
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| effective_target == get_stem(stems, pillar))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_stem_list_rule(
    name: &str,
    basis_prefix: &str,
    reference_branch: &str,
    rule_table: fn(&str) -> &'static [&'static str],
    stems: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let targets = rule_table(reference_branch);
    if targets.is_empty() {
        return;
    }
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| targets.contains(&get_stem(stems, pillar)))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_branch_rule(
    name: &str,
    basis_prefix: &str,
    reference_branch: &str,
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
    targets: (&str, &str, &str, &str),
) {
    let target_branch = match reference_branch {
        "申" | "子" | "辰" => targets.0,
        "寅" | "午" | "戌" => targets.1,
        "亥" | "卯" | "未" => targets.2,
        "巳" | "酉" | "丑" => targets.3,
        _ => "",
    };
    if target_branch.is_empty() {
        return;
    }
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| target_branch == get_branch(branches, pillar))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_direct_branch_rule(
    name: &str,
    basis_prefix: &str,
    reference_branch: &str,
    rule_table: fn(&str) -> Option<&'static str>,
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let Some(target_branch) = rule_table(reference_branch) else {
        return;
    };
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| target_branch == get_branch(branches, pillar))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        format!("{basis_prefix}{reference_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_special_branch_rule(
    name: &str,
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| matches!(get_branch(branches, pillar), "辰" | "巳" | "戌" | "亥"))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        name,
        "辰巳戌亥".to_string(),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_kong_wang_rule(
    response: &BaziResponse,
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let mut empty_branches = resolve_kong_wang_branches(&response.year_pillar.gan_zhi);
    for branch in resolve_kong_wang_branches(&response.day_pillar.gan_zhi) {
        push_unique(&mut empty_branches, branch);
    }
    if empty_branches.is_empty() {
        return;
    }
    let matched_pillars: Vec<&str> = PILLAR_ORDER
        .iter()
        .copied()
        .filter(|pillar| empty_branches.contains(&get_branch(branches, pillar).to_string()))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        "空亡",
        "年柱/日柱旬空".to_string(),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_tong_zi_rule(
    month_branch: &str,
    branches: &[(&str, &str)],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    let target_branches: &[&str] = match month_branch {
        "寅" | "卯" | "辰" | "申" | "酉" | "戌" => &["寅", "子"],
        "巳" | "午" | "未" | "亥" | "子" | "丑" => &["卯", "未", "辰"],
        _ => &[],
    };
    if target_branches.is_empty() {
        return;
    }
    let matched_pillars: Vec<&str> = ["day", "hour"]
        .iter()
        .copied()
        .filter(|pillar| target_branches.contains(&get_branch(branches, pillar)))
        .collect();
    if matched_pillars.is_empty() {
        return;
    }
    record_match(
        "童子煞",
        format!("月支{month_branch}"),
        matched_pillars,
        aggregated_matches,
        pillar_matches,
    );
}

fn add_kui_gang_rule(
    response: &BaziResponse,
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    if matches!(
        response.day_pillar.gan_zhi.as_str(),
        "壬辰" | "庚戌" | "庚辰" | "戊戌"
    ) {
        record_match(
            "魁罡",
            format!("日柱{}", response.day_pillar.gan_zhi),
            vec!["day"],
            aggregated_matches,
            pillar_matches,
        );
    }
}

fn add_fixed_day_pillar_rule(
    name: &str,
    response: &BaziResponse,
    rule_set: &[&str],
    aggregated_matches: &mut Vec<MutableMatch>,
    pillar_matches: &mut PillarMatches,
) {
    if rule_set.contains(&response.day_pillar.gan_zhi.as_str()) {
        record_match(
            name,
            format!("日柱{}", response.day_pillar.gan_zhi),
            vec!["day"],
            aggregated_matches,
            pillar_matches,
        );
    }
}

fn matches_mixed_target(target: &str, stem: &str, branch: &str) -> bool {
    is_stem(target)
        .then_some(stem == target)
        .unwrap_or(branch == target)
}

fn is_stem(target: &str) -> bool {
    matches!(
        target,
        "甲" | "乙" | "丙" | "丁" | "戊" | "己" | "庚" | "辛" | "壬" | "癸"
    )
}

fn resolve_kong_wang_branches(gan_zhi: &str) -> Vec<String> {
    if gan_zhi.chars().count() != 2 {
        return vec![];
    }
    let mut chars = gan_zhi.chars();
    let stem = chars.next().unwrap().to_string();
    let branch = chars.next().unwrap().to_string();
    let Some(stem_index) = stem_index(&stem) else {
        return vec![];
    };
    let Some(branch_index) = branch_sequence().iter().position(|item| *item == branch) else {
        return vec![];
    };
    let offset = ((branch_index as i32 - stem_index).rem_euclid(12)) as usize;
    let start_index = ((branch_index + 12 - offset) % 12) as usize;
    let empty_start = (start_index + 10) % 12;
    vec![
        branch_sequence()[empty_start].to_string(),
        branch_sequence()[(empty_start + 1) % 12].to_string(),
    ]
}

fn stem_index(stem: &str) -> Option<i32> {
    match stem {
        "甲" => Some(0),
        "乙" => Some(1),
        "丙" => Some(2),
        "丁" => Some(3),
        "戊" => Some(4),
        "己" => Some(5),
        "庚" => Some(6),
        "辛" => Some(7),
        "壬" => Some(8),
        "癸" => Some(9),
        _ => None,
    }
}

fn branch_sequence() -> &'static [&'static str] {
    &[
        "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
    ]
}

fn five_combination_partner(stem: &str) -> Option<&'static str> {
    match stem {
        "甲" => Some("己"),
        "己" => Some("甲"),
        "乙" => Some("庚"),
        "庚" => Some("乙"),
        "丙" => Some("辛"),
        "辛" => Some("丙"),
        "丁" => Some("壬"),
        "壬" => Some("丁"),
        "戊" => Some("癸"),
        "癸" => Some("戊"),
        _ => None,
    }
}

fn tian_yi_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" | "戊" | "庚" => &["丑", "未"],
        "乙" | "己" => &["子", "申"],
        "丙" | "丁" => &["亥", "酉"],
        "壬" | "癸" => &["卯", "巳"],
        "辛" => &["寅", "午"],
        _ => &[],
    }
}

fn wen_chang_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" => &["巳"],
        "乙" => &["午"],
        "丙" | "戊" => &["申"],
        "丁" | "己" => &["酉"],
        "庚" => &["亥"],
        "辛" => &["子"],
        "壬" => &["寅"],
        "癸" => &["卯"],
        _ => &[],
    }
}

fn tai_ji_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" | "乙" => &["子", "午"],
        "丙" | "丁" => &["卯", "酉"],
        "戊" | "己" => &["辰", "戌", "丑", "未"],
        "庚" | "辛" => &["寅", "亥"],
        "壬" | "癸" => &["巳", "申"],
        _ => &[],
    }
}

fn tian_de_by_month_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "寅" => Some("丁"),
        "卯" => Some("申"),
        "辰" => Some("壬"),
        "巳" => Some("辛"),
        "午" => Some("亥"),
        "未" => Some("甲"),
        "申" => Some("癸"),
        "酉" => Some("寅"),
        "戌" => Some("丙"),
        "亥" => Some("乙"),
        "子" => Some("巳"),
        "丑" => Some("庚"),
        _ => None,
    }
}

fn yue_de_by_month_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "寅" | "午" | "戌" => Some("丙"),
        "申" | "子" | "辰" => Some("壬"),
        "亥" | "卯" | "未" => Some("甲"),
        "巳" | "酉" | "丑" => Some("庚"),
        _ => None,
    }
}

fn de_xiu_by_month_branch(branch: &str) -> &'static [&'static str] {
    match branch {
        "寅" | "午" | "戌" => &["丙", "丁", "戊", "癸"],
        "申" | "子" | "辰" => &["壬", "癸", "戊", "己", "丙", "辛", "甲"],
        "巳" | "酉" | "丑" => &["庚", "辛", "乙"],
        "亥" | "卯" | "未" => &["甲", "乙", "丁", "壬"],
        _ => &[],
    }
}

fn hong_luan_by_year_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "子" => Some("卯"),
        "丑" => Some("寅"),
        "寅" => Some("丑"),
        "卯" => Some("子"),
        "辰" => Some("亥"),
        "巳" => Some("戌"),
        "午" => Some("酉"),
        "未" => Some("申"),
        "申" => Some("未"),
        "酉" => Some("午"),
        "戌" => Some("巳"),
        "亥" => Some("辰"),
        _ => None,
    }
}

fn tian_xi_by_year_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "子" => Some("酉"),
        "丑" => Some("申"),
        "寅" => Some("未"),
        "卯" => Some("午"),
        "辰" => Some("巳"),
        "巳" => Some("辰"),
        "午" => Some("卯"),
        "未" => Some("寅"),
        "申" => Some("丑"),
        "酉" => Some("子"),
        "戌" => Some("亥"),
        "亥" => Some("戌"),
        _ => None,
    }
}

fn gu_chen_by_year_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "亥" | "子" | "丑" => Some("寅"),
        "寅" | "卯" | "辰" => Some("巳"),
        "巳" | "午" | "未" => Some("申"),
        "申" | "酉" | "戌" => Some("亥"),
        _ => None,
    }
}

fn gua_su_by_year_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "亥" | "子" | "丑" => Some("辰"),
        "寅" | "卯" | "辰" => Some("丑"),
        "巳" | "午" | "未" => Some("戌"),
        "申" | "酉" | "戌" => Some("未"),
        _ => None,
    }
}

fn tian_chu_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" | "丙" => &["巳"],
        "乙" | "丁" => &["午"],
        "己" => &["酉"],
        "戊" => &["申"],
        "庚" => &["亥"],
        "壬" => &["寅"],
        "辛" => &["子"],
        "癸" => &["卯"],
        _ => &[],
    }
}

fn jin_yu_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" => &["辰"],
        "乙" => &["巳"],
        "丙" | "戊" => &["未"],
        "丁" | "己" => &["申"],
        "庚" => &["戌"],
        "辛" => &["亥"],
        "壬" => &["丑"],
        "癸" => &["寅"],
        _ => &[],
    }
}

fn yang_ren_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" => &["卯"],
        "乙" => &["寅"],
        "丙" | "戊" => &["午"],
        "丁" | "己" => &["巳"],
        "庚" => &["酉"],
        "辛" => &["申"],
        "壬" => &["子"],
        "癸" => &["亥"],
        _ => &[],
    }
}

fn guo_yin_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" => &["戌"],
        "乙" => &["亥"],
        "丙" | "戊" => &["丑"],
        "丁" | "己" => &["寅"],
        "庚" => &["辰"],
        "辛" => &["巳"],
        "壬" => &["未"],
        "癸" => &["申"],
        _ => &[],
    }
}

fn jie_sha_by_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "申" | "子" | "辰" => Some("巳"),
        "寅" | "午" | "戌" => Some("亥"),
        "亥" | "卯" | "未" => Some("申"),
        "巳" | "酉" | "丑" => Some("寅"),
        _ => None,
    }
}

fn zai_sha_by_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "申" | "子" | "辰" => Some("午"),
        "寅" | "午" | "戌" => Some("子"),
        "亥" | "卯" | "未" => Some("酉"),
        "巳" | "酉" | "丑" => Some("卯"),
        _ => None,
    }
}

fn wang_shen_by_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "申" | "子" | "辰" => Some("亥"),
        "寅" | "午" | "戌" => Some("申"),
        "亥" | "卯" | "未" => Some("寅"),
        "巳" | "酉" | "丑" => Some("巳"),
        _ => None,
    }
}

fn fu_xing_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" | "丙" => &["寅", "子"],
        "乙" | "癸" => &["丑", "卯"],
        "丁" => &["亥", "酉"],
        "戊" => &["申"],
        "己" => &["未"],
        "庚" => &["午"],
        "辛" => &["巳"],
        "壬" => &["辰"],
        _ => &[],
    }
}

fn fei_ren_by_stem(stem: &str) -> &'static [&'static str] {
    match stem {
        "甲" => &["酉"],
        "乙" => &["申"],
        "丙" | "戊" => &["子"],
        "丁" | "己" => &["亥"],
        "庚" => &["卯"],
        "辛" => &["寅"],
        "壬" => &["午"],
        "癸" => &["巳"],
        _ => &[],
    }
}

fn sang_men_by_year_branch(branch: &str) -> Option<&'static str> {
    match branch {
        "子" => Some("寅"),
        "丑" => Some("卯"),
        "寅" => Some("辰"),
        "卯" => Some("巳"),
        "辰" => Some("午"),
        "巳" => Some("未"),
        "午" => Some("申"),
        "未" => Some("酉"),
        "申" => Some("戌"),
        "酉" => Some("亥"),
        "戌" => Some("子"),
        "亥" => Some("丑"),
        _ => None,
    }
}
