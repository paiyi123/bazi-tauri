//! 地支沖、合的作用變化（草綠色講義第二十四章第一、二節，2026-10-21）。
//!
//! - 相沖：貼近（年月、月日、日時）沖力大，隔位小；得令、黨多者勝（得令者沖衰，衰者拔）；
//!   沖去忌神留住喜神為吉，沖去喜神留住忌神為凶；衰神沖旺，旺神發。
//! - 六合：地支有合無化，依本氣分生合（寅亥、辰酉、午未）與剋合（子丑、卯戌、巳申）；
//!   生合加強受生者、剋合削減受剋者，再看喜忌。六合不一定解沖。
//!
//! 只做結構判斷與喜忌標示，不改量化分數，也不回頭改強弱與喜忌方案。

use super::stem_interactions::{element_favor, Favor};
use super::*;

const SOURCE: &str = "2026-10-21";

const CLASHES: [(&str, &str); 6] = [
    ("子", "午"),
    ("卯", "酉"),
    ("寅", "申"),
    ("巳", "亥"),
    ("辰", "戌"),
    ("丑", "未"),
];

const SIX_COMBINATIONS: [(&str, &str); 6] = [
    ("子", "丑"),
    ("寅", "亥"),
    ("卯", "戌"),
    ("辰", "酉"),
    ("巳", "申"),
    ("午", "未"),
];

const STORAGE: [&str; 4] = ["辰", "戌", "丑", "未"];

fn pair_matches(table: &[(&str, &str)], a: &str, b: &str) -> bool {
    table
        .iter()
        .any(|(x, y)| (*x == a && *y == b) || (*x == b && *y == a))
}

fn branch_label(cycles: &[SixtyCycle; 4], index: usize) -> String {
    format!(
        "{}{}",
        BRANCH_LOCATION_LABELS[index],
        cycles[index].get_earth_branch().get_name()
    )
}

fn branch_label_with_god(cycles: &[SixtyCycle; 4], index: usize) -> String {
    let branch = cycles[index].get_earth_branch();
    let element = branch.get_element();
    let day = cycles[2].get_heaven_stem().get_element();
    format!(
        "{}（{}，{}）",
        branch_label(cycles, index),
        element_name(&element),
        family_for_element(&day, &element)
    )
}

fn main_element(cycles: &[SixtyCycle; 4], index: usize) -> Element {
    cycles[index].get_earth_branch().get_element()
}

fn distance_text(left: usize, right: usize) -> (&'static str, &'static str) {
    if right - left == 1 {
        ("貼近", "沖力較大")
    } else if right - left == 2 {
        ("隔一位", "沖力較小")
    } else {
        ("遙隔", "沖力更小")
    }
}

/// 黨眾：本氣同五行的地支數，月令（得令）再加一；同分時得令者勝。
fn side_power(cycles: &[SixtyCycle; 4], element: &Element) -> f64 {
    let count = (0..4)
        .filter(|index| main_element(cycles, *index) == *element)
        .count() as f64;
    let in_season = if main_element(cycles, 1) == *element { 1.0 } else { 0.0 };
    count + in_season
}

/// 敗方得生：同柱天干或任一地支本氣為生敗方之五行（例七：亥得辛、申生）。
fn loser_supports(cycles: &[SixtyCycle; 4], index: usize) -> Vec<String> {
    let element = main_element(cycles, index);
    let generator = element.get_reinforced();
    let mut supports = Vec::new();
    if cycles[index].get_heaven_stem().get_element() == generator {
        supports.push(format!(
            "{}{}",
            STEM_LOCATION_LABELS[index],
            cycles[index].get_heaven_stem().get_name()
        ));
    }
    for other in 0..4 {
        if main_element(cycles, other) == generator {
            supports.push(branch_label(cycles, other));
        }
    }
    supports
}

fn favor_code(favor: Favor) -> &'static str {
    match favor {
        Favor::Good => "good",
        Favor::Bad => "bad",
        Favor::Mixed => "mixed",
        Favor::Neutral => "neutral",
    }
}

fn favor_label(favor: Favor) -> &'static str {
    match favor {
        Favor::Good => "喜用",
        Favor::Bad => "忌",
        Favor::Mixed => "依方案而定",
        Favor::Neutral => "未列喜忌",
    }
}

fn clash_entry(
    cycles: &[SixtyCycle; 4],
    left: usize,
    right: usize,
    favor: &dyn Fn(&str) -> Favor,
) -> LessonBranchClash {
    let (distance, force) = distance_text(left, right);
    let left_element = main_element(cycles, left);
    let right_element = main_element(cycles, right);
    let left_power = side_power(cycles, &left_element);
    let right_power = side_power(cycles, &right_element);
    let names = format!(
        "{}{}",
        cycles[left].get_earth_branch().get_name(),
        cycles[right].get_earth_branch().get_name()
    );
    let mut notes = Vec::new();
    let both_storage = STORAGE.contains(&cycles[left].get_earth_branch().get_name().as_str())
        && STORAGE.contains(&cycles[right].get_earth_branch().get_name().as_str());

    let power_note = format!(
        "黨眾（本氣同五行地支數，得令另加一）：{} {}、{} {}。",
        element_name(&left_element),
        left_power,
        element_name(&right_element),
        right_power
    );

    if both_storage {
        // 四庫之沖：本氣都是土，是餘氣沖餘氣，不分勝負。
        let earth = Element::from_name("土");
        let earth_power = side_power(cycles, &earth);
        let touches_month = (left, right) == (0, 1) || (left, right) == (1, 2);
        let day_earth = cycles[2].get_heaven_stem().get_element() == earth;
        let (verdict, verdict_label, reason) = if touches_month || earth_power >= 3.0 {
            let effect = favor(&element_name(&earth));
            (
                match effect {
                    Favor::Good => Favor::Good,
                    Favor::Bad => Favor::Bad,
                    other => other,
                },
                format!("土沖土愈旺（土為{}）", favor_label(effect)),
                format!(
                    "{}墓庫相沖，是餘氣沖餘氣、庫神戰庫神，各有損傷；{}，土星無損反而愈旺。",
                    names,
                    if touches_month {
                        "沖在年月或月日（碰到月令）".to_string()
                    } else {
                        format!("土黨多（{}）", earth_power)
                    }
                ),
            )
        } else {
            (
                Favor::Mixed,
                "土氣動搖".to_string(),
                format!(
                    "{}墓庫相沖，土不得令又不黨多，沖在{}，土的根基反而動搖；餘氣（{}）各有損傷。",
                    names,
                    if left == 2 && right == 3 { "日時" } else { "隔位" },
                    [left, right]
                        .iter()
                        .map(|index| {
                            cycles[*index]
                                .get_earth_branch()
                                .get_hide_heaven_stems()
                                .iter()
                                .skip(1)
                                .map(|hidden| hidden.get_heaven_stem().get_name())
                                .collect::<Vec<_>>()
                                .join("")
                        })
                        .collect::<Vec<_>>()
                        .join("、")
                ),
            )
        };
        if day_earth && !touches_month && earth_power < 3.0 {
            notes.push("日主屬土而土不旺時，被沖的若是日主唯一的根，為不吉之兆（講義：戊日時柱壬戌逢辰沖）。".to_string());
        }
        notes.push("「墓庫逢沖則發」「土沖土愈旺」都不可執一，要看得令與黨眾。".to_string());
        return LessonBranchClash {
            id: format!("branch-clash-{}-{}", left + 1, right + 1),
            branches: names,
            positions: vec![branch_label_with_god(cycles, left), branch_label_with_god(cycles, right)],
            distance: distance.to_string(),
            force_label: force.to_string(),
            winner: None,
            loser: None,
            outcome: "storage".to_string(),
            outcome_label: "四庫相沖".to_string(),
            verdict: favor_code(verdict).to_string(),
            verdict_label,
            reason,
            notes,
            source_date: SOURCE.to_string(),
        };
    }

    let (winner, loser) = if left_power > right_power {
        (left, right)
    } else if right_power > left_power {
        (right, left)
    } else if right == 1 {
        (right, left)
    } else {
        (left, right)
    };
    let winner_power = left_power.max(right_power);
    let loser_power = left_power.min(right_power);
    let winner_element = main_element(cycles, winner);
    let loser_element = main_element(cycles, loser);
    let winner_favor = favor(&element_name(&winner_element));
    let loser_favor = favor(&element_name(&loser_element));
    let tie_note = if (left_power - right_power).abs() < f64::EPSILON {
        "兩邊黨眾相同，得令者勝（得令者沖衰）。".to_string()
    } else {
        String::new()
    };
    let supports = loser_supports(cycles, loser);
    let both_damaged = supports.len() >= 2;
    let overwhelming = loser_power > 0.0 && winner_power >= loser_power * 2.0;

    let (outcome, outcome_label, verdict, verdict_label, effect) = if both_damaged {
        let verdict = match (loser_favor, winner_favor) {
            (Favor::Good, Favor::Bad) | (Favor::Bad, Favor::Good) => Favor::Mixed,
            (Favor::Bad, _) => Favor::Good,
            (Favor::Good, _) => Favor::Bad,
            _ => Favor::Neutral,
        };
        let label = match (loser_favor, winner_favor) {
            (Favor::Good, Favor::Bad) | (Favor::Bad, Favor::Good) => {
                "各有損傷：雖沖去忌神，喜神亦傷".to_string()
            }
            _ => "各有損傷".to_string(),
        };
        (
            "both_damaged",
            "各有損傷",
            verdict,
            label,
            format!(
                "{}較旺，但{}得{}生扶，兩邊交戰各有損傷；須行運補足。",
                branch_label(cycles, winner),
                branch_label(cycles, loser),
                supports.join("、")
            ),
        )
    } else {
        let (verdict, label) = match loser_favor {
            _ if overwhelming && winner_favor == Favor::Bad => (Favor::Bad, "衰神沖旺，旺神發，凶"),
            Favor::Good => (Favor::Bad, "去喜存忌，凶"),
            Favor::Bad => (Favor::Good, "去忌存喜，吉"),
            Favor::Mixed => (Favor::Mixed, "依方案而定"),
            Favor::Neutral => (Favor::Neutral, "未列喜忌"),
        };
        (
            "decisive",
            "勝負分明",
            verdict,
            label.to_string(),
            format!(
                "{}勝、{}敗（{}為{}、{}為{}）。",
                branch_label(cycles, winner),
                branch_label(cycles, loser),
                element_name(&loser_element),
                favor_label(loser_favor),
                element_name(&winner_element),
                favor_label(winner_favor)
            ),
        )
    };
    if overwhelming {
        notes.push(format!(
            "衰神沖旺，旺神發：{}去沖旺{}，杯水車薪反激其勢，逆其旺氣禍害必起{}。",
            element_name(&loser_element),
            element_name(&winner_element),
            if winner_favor == Favor::Bad { "（旺神為忌，更凶）" } else { "" }
        ));
    }
    if !tie_note.is_empty() {
        notes.push(tie_note);
    }
    notes.push(power_note);
    let palace = |index: usize| match index {
        0 => "祖上宮",
        1 => "父母宮",
        2 => "夫妻宮",
        _ => "子女宮",
    };
    notes.push(format!(
        "宮位：{}沖{}（例：日支喜神被月支沖，可聯想婆媳不合）。",
        palace(left),
        palace(right)
    ));
    let reason = format!("{}相沖（{}，{}）。{}", names, distance, force, effect);
    LessonBranchClash {
        id: format!("branch-clash-{}-{}", left + 1, right + 1),
        branches: names,
        positions: vec![branch_label_with_god(cycles, left), branch_label_with_god(cycles, right)],
        distance: distance.to_string(),
        force_label: force.to_string(),
        winner: Some(branch_label(cycles, winner)),
        loser: Some(branch_label(cycles, loser)),
        outcome: outcome.to_string(),
        outcome_label: outcome_label.to_string(),
        verdict: favor_code(verdict).to_string(),
        verdict_label,
        reason,
        notes,
        source_date: SOURCE.to_string(),
    }
}

/// 六合依本氣：一方生另一方為生合，一方剋另一方為剋合。
fn combination_kind(cycles: &[SixtyCycle; 4], left: usize, right: usize) -> (bool, usize, usize) {
    let a = main_element(cycles, left);
    let b = main_element(cycles, right);
    if a.get_reinforce() == b {
        (true, left, right)
    } else if b.get_reinforce() == a {
        (true, right, left)
    } else if a.get_restrain() == b {
        (false, left, right)
    } else {
        (false, right, left)
    }
}

fn combination_entry(
    cycles: &[SixtyCycle; 4],
    left: usize,
    right: usize,
    clashes: &[(usize, usize)],
    favor: &dyn Fn(&str) -> Favor,
) -> LessonBranchCombination {
    let names = format!(
        "{}{}",
        cycles[left].get_earth_branch().get_name(),
        cycles[right].get_earth_branch().get_name()
    );
    let (generating, actor, target) = combination_kind(cycles, left, right);
    let actor_element = main_element(cycles, actor);
    let target_element = main_element(cycles, target);
    let mut notes = Vec::new();
    let adjacent = right - left == 1;

    let (kind, kind_label, verdict, verdict_label, effect) = if generating {
        let target_favor = favor(&element_name(&target_element));
        let (verdict, label) = match target_favor {
            Favor::Good => (Favor::Good, "加強喜神，吉"),
            Favor::Bad => (Favor::Bad, "加強忌神，凶"),
            Favor::Mixed => (Favor::Mixed, "依方案而定"),
            Favor::Neutral => (Favor::Neutral, "未列喜忌"),
        };
        (
            "generating",
            "生合",
            verdict,
            label.to_string(),
            format!(
                "{}合為{}生{}，加強{}（{}）之力，{}的性質依然存在。",
                names,
                element_name(&actor_element),
                element_name(&target_element),
                element_name(&target_element),
                favor_label(target_favor),
                cycles[actor].get_earth_branch().get_name()
            ),
        )
    } else {
        let target_favor = favor(&element_name(&target_element));
        let actor_favor = favor(&element_name(&actor_element));
        let (verdict, label) = match target_favor {
            Favor::Good => (Favor::Bad, "削減喜神，凶".to_string()),
            Favor::Bad => {
                if actor_favor == Favor::Bad {
                    (Favor::Good, "削減忌神，但剋者亦忌，效果有限".to_string())
                } else {
                    (Favor::Good, "削減忌神，吉".to_string())
                }
            }
            Favor::Mixed => (Favor::Mixed, "依方案而定".to_string()),
            Favor::Neutral => (Favor::Neutral, "未列喜忌".to_string()),
        };
        (
            "controlling",
            "剋合",
            verdict,
            label,
            format!(
                "{}合為{}剋{}，削減{}（{}）之力，但{}仍是{}，不因合而消失。",
                names,
                element_name(&actor_element),
                element_name(&target_element),
                element_name(&target_element),
                favor_label(target_favor),
                cycles[target].get_earth_branch().get_name(),
                element_name(&target_element)
            ),
        )
    };

    // 六合與相沖並見：合神加強沖方 → 加重沖力；剋合絆住沖方 → 減低沖力；沖方貪生合神 → 減低沖力。
    let mut effects: Vec<String> = Vec::new();
    for (a, b) in clashes {
        for (member, partner_in_combo) in [(left, right), (right, left)] {
            let other = if *a == member {
                *b
            } else if *b == member {
                *a
            } else {
                continue;
            };
            if other == partner_in_combo {
                continue;
            }
            let member_name = branch_label(cycles, member);
            let other_name = branch_label(cycles, other);
            let partner_name = branch_label(cycles, partner_in_combo);
            let text = if main_element(cycles, other).get_reinforce() == main_element(cycles, partner_in_combo) {
                format!(
                    "{}貪生{}，減低對{}的沖力；在原局只能減沖，在大運是通關解沖，運過恢復沖力。",
                    other_name, partner_name, member_name
                )
            } else if generating && target == member {
                format!(
                    "{}生{}，增加{}與{}相沖之力：六合不但不能解沖，反而加重沖力。",
                    partner_name, member_name, member_name, other_name
                )
            } else if !generating && target == member {
                format!(
                    "{}剋合絆住{}，減低{}與{}相沖之力（只是減低，不是完全解沖）。",
                    partner_name, member_name, member_name, other_name
                )
            } else {
                format!(
                    "{}被{}合住，與{}相沖之力略有牽制；六合不一定解沖。",
                    member_name, partner_name, other_name
                )
            };
            effects.push(text);
        }
    }
    effects.dedup();
    let clash_effect = if effects.is_empty() { None } else { Some(effects.join("")) };
    if !adjacent {
        notes.push("講義六合例題都是相鄰兩支；隔位六合的力量老師沒有另外說明，僅供參考。".to_string());
    }
    notes.push("地支有合無化：不採「辰酉合化金」「寅亥合化木」等說法（講義第 25–26 頁）。".to_string());

    LessonBranchCombination {
        id: format!("branch-combination-{}-{}", left + 1, right + 1),
        branches: names,
        positions: vec![branch_label_with_god(cycles, left), branch_label_with_god(cycles, right)],
        kind: kind.to_string(),
        kind_label: kind_label.to_string(),
        adjacent,
        verdict: favor_code(verdict).to_string(),
        verdict_label,
        reason: effect,
        clash_effect,
        notes,
        source_date: SOURCE.to_string(),
    }
}

pub(super) fn classify_branch_interactions(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> (Vec<LessonBranchClash>, Vec<LessonBranchCombination>) {
    let names = cycles
        .iter()
        .map(|cycle| cycle.get_earth_branch().get_name())
        .collect::<Vec<_>>();
    let mut clash_pairs = Vec::new();
    let mut combo_pairs = Vec::new();
    for left in 0..4 {
        for right in left + 1..4 {
            if pair_matches(&CLASHES, &names[left], &names[right]) {
                clash_pairs.push((left, right));
            }
            if pair_matches(&SIX_COMBINATIONS, &names[left], &names[right]) {
                combo_pairs.push((left, right));
            }
        }
    }
    // 兩個同字沖一個（如雙午沖子）只列最近的一組，力量已算進黨眾。
    let mut kept: Vec<(usize, usize)> = Vec::new();
    for (left, right) in clash_pairs.iter().copied() {
        let key = |a: usize, b: usize| {
            let mut pair = [names[a].clone(), names[b].clone()];
            pair.sort();
            pair
        };
        match kept.iter().position(|(a, b)| key(*a, *b) == key(left, right)) {
            Some(position) => {
                let (a, b) = kept[position];
                if right - left < b - a {
                    kept[position] = (left, right);
                }
            }
            None => kept.push((left, right)),
        }
    }
    let clashes = kept
        .iter()
        .map(|(left, right)| clash_entry(cycles, *left, *right, favor))
        .collect();
    let combinations = combo_pairs
        .iter()
        .map(|(left, right)| combination_entry(cycles, *left, *right, &kept, favor))
        .collect();
    (clashes, combinations)
}

pub(super) fn build_branch_interactions(
    natal_cycles: &[SixtyCycle; 4],
    preferences: &LessonPreferenceAnalysis,
) -> LessonBranchInteractionAnalysis {
    let favor = |element: &str| element_favor(preferences, element);
    let (clashes, combinations) = classify_branch_interactions(natal_cycles, &favor);
    let gatherings = super::branch_gatherings::classify_gatherings(natal_cycles, &favor);
    let punishments = super::branch_gatherings::classify_punishments(natal_cycles, &favor);
    let mut parts = Vec::new();
    for item in &clashes {
        parts.push(format!("{}沖（{}）", item.branches, item.verdict_label));
    }
    for item in &combinations {
        parts.push(format!("{}{}（{}）", item.branches, item.kind_label, item.verdict_label));
    }
    for item in &gatherings {
        parts.push(format!("{}（{}）", item.kind_label, item.verdict_label));
    }
    for item in &punishments {
        parts.push(format!("{}：{}（{}）", item.kind_label, item.relation, item.verdict_label));
    }
    let conclusion = if parts.is_empty() {
        "地支沒有沖、合、會、刑、害。".to_string()
    } else {
        format!("{}。", parts.join("；"))
    };
    let status = if preferences.plans.len() > 1 {
        "conditional"
    } else {
        "candidate"
    };
    LessonBranchInteractionAnalysis {
        title: "地支沖合會刑害".to_string(),
        conclusion,
        status: status.to_string(),
        clashes,
        combinations,
        gatherings,
        punishments,
        limitations: vec![
            "勝負用「本氣同五行的地支數，得令再加一」粗估；天干透出、半合三合的助力沒有計入，與老師逐例的文字判斷可能不同。".to_string(),
            "敗方有兩處以上生扶（同柱天干或地支本氣）時判為各有損傷，這是依講義例七整理的工程門檻。".to_string(),
            "三合、三會、刑害（講義第三至五節）課堂還沒講，依講義整理：成局條件是依第 26 章例題歸納的推論；刑害只依本氣生剋論，不論刑害本身吉凶（老師：害屬神煞）。".to_string(),
            "沖合結果沒有回頭重算強弱與喜忌；量化五行力分數不受影響。".to_string(),
        ],
        sources: vec![SOURCE.to_string(), super::branch_gatherings::SOURCE_TEXTBOOK.to_string()],
    }
}
