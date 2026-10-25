//! 地支三合、三會、刑害的作用變化（草綠色講義第二十四章第三～五節，第 30–41 頁）。
//!
//! 講義（書明說）：
//! - 三合、三會只論合力大小、會成氣或不成氣，不能論化；合神居月令、月干透出者力最大，
//!   其次年時透，再次合神居月令而不透；合神不居月令、局中又有他字間之，依五行常理論。
//! - 半合（含中神）同理；缺中神的半合（亥未、寅戌、巳丑、申辰）須歲運填實才增合力。
//! - 吉凶看所合成的五行對日主是喜是忌。辰戌丑未全也可論土方土局。
//! - 刑害傳統論法近乎神煞不足憑，改依五行本氣生剋制化配合喜忌論。
//!
//! 工程推論（講義沒有逐字寫，依第 26 章例題歸納）：成局要「月支在局中，且月支本氣就是局的
//! 五行，或月干透出局的五行」；三會另接受三字連珠順排含月令（26 章地支例三寅卯辰）。
//! 半合若不含中神的那一支居月令且剋局的五行（巳酉半合、巳居月令），改依本氣生剋論（26 章地支例七）。

use super::stem_interactions::Favor;
use super::*;

pub(super) const SOURCE_TEXTBOOK: &str = "講義第24章";

const THREE_COMBINATIONS: [([&str; 3], &str, &str); 4] = [
    (["申", "子", "辰"], "子", "水"),
    (["亥", "卯", "未"], "卯", "木"),
    (["寅", "午", "戌"], "午", "火"),
    (["巳", "酉", "丑"], "酉", "金"),
];

const THREE_MEETINGS: [([&str; 3], &str, &str, &str); 4] = [
    (["寅", "卯", "辰"], "卯", "木", "東方"),
    (["巳", "午", "未"], "午", "火", "南方"),
    (["申", "酉", "戌"], "酉", "金", "西方"),
    (["亥", "子", "丑"], "子", "水", "北方"),
];

const PUNISHMENT_GROUPS: [&[&str]; 3] = [&["子", "卯"], &["寅", "巳", "申"], &["丑", "戌", "未"]];
const SELF_PUNISHMENTS: [&str; 4] = ["辰", "午", "酉", "亥"];
const HARMS: [(&str, &str); 6] = [
    ("子", "未"),
    ("丑", "午"),
    ("寅", "巳"),
    ("卯", "辰"),
    ("申", "亥"),
    ("酉", "戌"),
];

fn names(cycles: &[SixtyCycle; 4]) -> Vec<String> {
    cycles
        .iter()
        .map(|cycle| cycle.get_earth_branch().get_name())
        .collect()
}

fn label(cycles: &[SixtyCycle; 4], index: usize) -> String {
    format!(
        "{}{}",
        BRANCH_LOCATION_LABELS[index],
        cycles[index].get_earth_branch().get_name()
    )
}

fn main_element(cycles: &[SixtyCycle; 4], index: usize) -> Element {
    cycles[index].get_earth_branch().get_element()
}

fn stem_element(cycles: &[SixtyCycle; 4], index: usize) -> Element {
    cycles[index].get_heaven_stem().get_element()
}

pub(super) fn favor_code(favor: Favor) -> &'static str {
    match favor {
        Favor::Good => "good",
        Favor::Bad => "bad",
        Favor::Mixed => "mixed",
        Favor::Neutral => "neutral",
    }
}

pub(super) fn favor_label(favor: Favor) -> &'static str {
    match favor {
        Favor::Good => "喜用",
        Favor::Bad => "忌",
        Favor::Mixed => "依方案而定",
        Favor::Neutral => "未列喜忌",
    }
}

fn invert(favor: Favor) -> Favor {
    match favor {
        Favor::Good => Favor::Bad,
        Favor::Bad => Favor::Good,
        other => other,
    }
}

fn combine(favors: &[Favor]) -> Favor {
    let good = favors.contains(&Favor::Good);
    let bad = favors.contains(&Favor::Bad);
    if favors.contains(&Favor::Mixed) || (good && bad) {
        Favor::Mixed
    } else if good {
        Favor::Good
    } else if bad {
        Favor::Bad
    } else {
        Favor::Neutral
    }
}

/// 局神在月干、年干或時干透出的位置（日干是日主本身，不算）。
fn transparent_positions(cycles: &[SixtyCycle; 4], element: &Element) -> Vec<usize> {
    [1usize, 0, 3]
        .into_iter()
        .filter(|index| stem_element(cycles, *index) == *element)
        .collect()
}

/// 局中有沒有他字間隔：成員最左與最右之間夾著非成員的地支。
fn separated_by(names: &[String], members: &[usize], set: &[&str]) -> Option<usize> {
    let first = *members.iter().min()?;
    let last = *members.iter().max()?;
    (first + 1..last).find(|index| !set.contains(&names[*index].as_str()))
}

/// 三字連珠順排：三個成員剛好佔三個相連的柱，且依序排列（順或逆），並含月令。
fn in_sequence(names: &[String], set: &[&str]) -> bool {
    for start in 0..2 {
        let window = [&names[start], &names[start + 1], &names[start + 2]];
        if !(start..start + 3).contains(&1) {
            continue;
        }
        let forward = window.iter().zip(set.iter()).all(|(a, b)| a.as_str() == *b);
        let backward = window.iter().zip(set.iter().rev()).all(|(a, b)| a.as_str() == *b);
        if forward || backward {
            return true;
        }
    }
    false
}

fn gathering_force(
    cycles: &[SixtyCycle; 4],
    names: &[String],
    center: &str,
    element: &Element,
) -> (String, Vec<usize>) {
    let transparent = transparent_positions(cycles, element);
    let center_in_month = names[1] == center;
    let month_transparent = transparent.contains(&1);
    let text = if center_in_month && month_transparent {
        "合力最大（合神居月令、月干透出）".to_string()
    } else if !transparent.is_empty() {
        format!(
            "合力次之（{}透{}）",
            transparent
                .iter()
                .map(|index| STEM_LOCATION_LABELS[*index])
                .collect::<Vec<_>>()
                .join("、"),
            element_name(element)
        )
    } else if center_in_month {
        "合力再次（合神居月令而不透干）".to_string()
    } else if main_element(cycles, 1) == *element {
        format!(
            "月令{}本氣屬{}，局成但合神不透",
            names[1],
            element_name(element)
        )
    } else {
        "三字連珠含月令，合神不透（推論）".to_string()
    };
    (text, transparent)
}

fn verdict_for_element(favor: Favor, element: &str, noun: &str) -> (Favor, String) {
    match favor {
        Favor::Good => (Favor::Good, format!("{}成{}為喜，論吉", noun, element)),
        Favor::Bad => (Favor::Bad, format!("{}成{}為忌，論凶", noun, element)),
        Favor::Mixed => (Favor::Mixed, "依方案而定".to_string()),
        Favor::Neutral => (Favor::Neutral, "未列喜忌".to_string()),
    }
}

fn full_gathering(
    cycles: &[SixtyCycle; 4],
    names: &[String],
    set: &[&str; 3],
    center: &str,
    element_text: &str,
    meeting: Option<&str>,
    favor: &dyn Fn(&str) -> Favor,
) -> Option<LessonBranchGathering> {
    if !set.iter().all(|item| names.iter().any(|name| name == item)) {
        return None;
    }
    let members = (0..4)
        .filter(|index| set.contains(&names[*index].as_str()))
        .collect::<Vec<_>>();
    let element = Element::from_name(element_text);
    let kind_name = if meeting.is_some() { "三會" } else { "三合" };
    let local_name = match meeting {
        Some(direction) => format!("{}{}", direction, element_text),
        None => format!("{}局", element_text),
    };
    let set_text = set.join("");
    let positions = members.iter().map(|index| label(cycles, *index)).collect::<Vec<_>>();
    let month_in = members.contains(&1);
    let month_main = main_element(cycles, 1) == element;
    let month_transparent = stem_element(cycles, 1) == element;
    let sequence = meeting.is_some() && in_sequence(names, set);
    let separated = separated_by(names, &members, set);
    let mut notes = Vec::new();
    let formed = separated.is_none() && month_in && (month_main || month_transparent || sequence);
    let source_page = if meeting.is_some() { "講義第 34 頁" } else { "講義第 30 頁" };

    if !formed {
        let why = if let Some(index) = separated {
            format!("{}夾在中間，局中有他字間之", label(cycles, index))
        } else if !month_in {
            "合神不居月令".to_string()
        } else {
            format!(
                "月令{}本氣不是{}、月干也沒透{}",
                names[1], element_text, element_text
            )
        };
        notes.push(format!(
            "{}只論{}，不能論化；不成局時依五行常理論生剋制化（{}）。",
            kind_name,
            if meeting.is_some() { "會成氣或不成氣" } else { "合力大小" },
            source_page
        ));
        notes.push("成局條件「月支在局中且本氣同局或月干透出」是依第 26 章例題歸納的推論。".to_string());
        return Some(LessonBranchGathering {
            id: format!("branch-gathering-{}-{}", if meeting.is_some() { "meeting" } else { "triple" }, set_text),
            kind: if meeting.is_some() { "three_meeting" } else { "three_combination" }.to_string(),
            kind_label: format!("{}{}（不成局）", set_text, kind_name),
            branches: set_text.clone(),
            positions,
            element: element_text.to_string(),
            formed: false,
            force_label: "依常理論".to_string(),
            verdict: "neutral".to_string(),
            verdict_label: "不論合局".to_string(),
            reason: format!("{}三字俱全，但{}，不以{}論，各支照本氣生剋看。", set_text, why, local_name),
            notes,
            source_date: SOURCE_TEXTBOOK.to_string(),
        });
    }

    let (force, _) = gathering_force(cycles, names, center, &element);
    let (verdict, verdict_label) = verdict_for_element(favor(element_text), element_text, if meeting.is_some() { "會" } else { "合" });
    let others = members
        .iter()
        .filter(|index| main_element(cycles, **index) != element)
        .map(|index| format!("{}（{}）", names[*index], element_name(&main_element(cycles, *index))))
        .collect::<Vec<_>>();
    if !others.is_empty() {
        notes.push(format!(
            "{}仍保留本氣，只是氣被局神牽引（講義第 26 章：「非月令戌土化為火，戌仍是戌」「戊土之氣盡洩於金，但非化為金」）。",
            others.join("、")
        ));
    }
    if sequence && !month_main && !month_transparent {
        notes.push(format!(
            "月令{}本氣不是{}、也沒透干，依「三字連珠」順排含月令仍論成方（講義第 26 章地支例三：寅卯辰三會木方增強木力），屬推論。",
            names[1], element_text
        ));
    }
    notes.push(format!(
        "{}只論{}，不能論化（{}）。",
        kind_name,
        if meeting.is_some() { "會成氣或不成氣" } else { "合力大小" },
        source_page
    ));
    Some(LessonBranchGathering {
        id: format!("branch-gathering-{}-{}", if meeting.is_some() { "meeting" } else { "triple" }, set_text),
        kind: if meeting.is_some() { "three_meeting" } else { "three_combination" }.to_string(),
        kind_label: format!("{}{}{}", set_text, kind_name, local_name),
        branches: set_text.clone(),
        positions,
        element: element_text.to_string(),
        formed: true,
        force_label: force,
        verdict: favor_code(verdict).to_string(),
        verdict_label,
        reason: format!(
            "{}{}，{}之力增強（{}為{}）。",
            set_text,
            if meeting.is_some() { "會成" } else { "合成" },
            element_text,
            element_text,
            favor_label(favor(element_text))
        ),
        notes,
        source_date: SOURCE_TEXTBOOK.to_string(),
    })
}

fn half_gatherings(
    cycles: &[SixtyCycle; 4],
    names: &[String],
    favor: &dyn Fn(&str) -> Favor,
    covered: &[&str],
) -> Vec<LessonBranchGathering> {
    let mut list = Vec::new();
    for (set, center, element_text) in THREE_COMBINATIONS {
        if covered.contains(&set.join("").as_str()) {
            continue;
        }
        let element = Element::from_name(element_text);
        for left in 0..3 {
            let right = left + 1;
            let (a, b) = (names[left].as_str(), names[right].as_str());
            if a == b || !set.contains(&a) || !set.contains(&b) {
                continue;
            }
            let pair = format!("{}{}", a, b);
            let positions = vec![label(cycles, left), label(cycles, right)];
            let has_center = a == center || b == center;
            if !has_center {
                // 缺中神（亥未、寅戌、巳丑、申辰）：講義說在命以常理論，須歲運填實合神。
                list.push(LessonBranchGathering {
                    id: format!("branch-gathering-gap-{}-{}", left + 1, right + 1),
                    kind: "incomplete_half".to_string(),
                    kind_label: format!("{}缺中神", pair),
                    branches: pair.clone(),
                    positions,
                    element: element_text.to_string(),
                    formed: false,
                    force_label: format!("須歲運見{}填實", center),
                    verdict: "neutral".to_string(),
                    verdict_label: "在命以常理論".to_string(),
                    reason: format!(
                        "{}缺中神{}，原局不論合；大運流年見{}填實，才增其合力成{}局。",
                        pair, center, center, element_text
                    ),
                    notes: vec!["講義第 30 頁：如地支申辰，大運行子，謂之填實。".to_string()],
                    source_date: SOURCE_TEXTBOOK.to_string(),
                });
                continue;
            }
            let other = if a == center { right } else { left };
            let other_element = main_element(cycles, other);
            let mut notes = Vec::new();
            // 非中神的一支居月令，且其本氣剋局的五行：依本氣生剋論（26 章地支例七 巳酉半合「亦為火剋金（火居月令）」）。
            if other == 1 && other_element.get_restrain() == element {
                let verdict = invert(favor(element_text));
                notes.push("講義第 26 章地支例七：巳酉半合，巳居月令 →「亦為火剋金（火居月令）」；此處依同理推論。".to_string());
                list.push(LessonBranchGathering {
                    id: format!("branch-gathering-half-{}-{}", left + 1, right + 1),
                    kind: "half_combination".to_string(),
                    kind_label: format!("{}半合（依本氣生剋）", pair),
                    branches: pair.clone(),
                    positions,
                    element: element_text.to_string(),
                    formed: false,
                    force_label: format!("{}居月令", names[other]),
                    verdict: favor_code(verdict).to_string(),
                    verdict_label: match verdict {
                        Favor::Good => format!("{}剋{}，削減忌神", element_name(&other_element), element_text),
                        Favor::Bad => format!("{}剋{}，削減喜神", element_name(&other_element), element_text),
                        _ => favor_label(verdict).to_string(),
                    },
                    reason: format!(
                        "{}半合，但{}得月令、本氣{}剋{}，不論{}局，論{}剋{}（{}為{}）。",
                        pair,
                        names[other],
                        element_name(&other_element),
                        element_text,
                        element_text,
                        element_name(&other_element),
                        element_text,
                        element_text,
                        favor_label(favor(element_text))
                    ),
                    notes,
                    source_date: SOURCE_TEXTBOOK.to_string(),
                });
                continue;
            }
            let (force, _) = gathering_force(cycles, names, center, &element);
            let force = if names[1] != center && transparent_positions(cycles, &element).is_empty() {
                "半合".to_string()
            } else {
                force
            };
            let (verdict, verdict_label) = verdict_for_element(favor(element_text), element_text, "半合");
            if other_element != element {
                notes.push(format!(
                    "{}（{}）的氣被{}局牽引：生{}者洩、被{}剋者減，但{}仍保留本氣（半合不論化）。",
                    names[other],
                    element_name(&other_element),
                    element_text,
                    element_text,
                    element_text,
                    names[other]
                ));
            }
            list.push(LessonBranchGathering {
                id: format!("branch-gathering-half-{}-{}", left + 1, right + 1),
                kind: "half_combination".to_string(),
                kind_label: format!("{}半合{}局", pair, element_text),
                branches: pair.clone(),
                positions,
                element: element_text.to_string(),
                formed: true,
                force_label: force,
                verdict: favor_code(verdict).to_string(),
                verdict_label,
                reason: format!(
                    "{}半合{}局，{}之力增強（{}為{}）。",
                    pair,
                    element_text,
                    element_text,
                    element_text,
                    favor_label(favor(element_text))
                ),
                notes,
                source_date: SOURCE_TEXTBOOK.to_string(),
            });
        }
    }
    list
}

pub(super) fn classify_gatherings(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> Vec<LessonBranchGathering> {
    let names = names(cycles);
    let mut list = Vec::new();
    let mut covered: Vec<&str> = Vec::new();
    for (set, center, element) in THREE_COMBINATIONS.iter() {
        if let Some(item) = full_gathering(cycles, &names, set, center, element, None, favor) {
            covered.push(match *element {
                "水" => "申子辰",
                "木" => "亥卯未",
                "火" => "寅午戌",
                _ => "巳酉丑",
            });
            // 不成局的三合：合神居月令時，相鄰兩支仍可論半合（26 章地支例八：亥卯未被巳隔開，只論卯未半合）；
            // 合神不居月令（旺神不當令）則整組依常理論，不再拆半合（第三節例四、例五；26 章地支例十七）。
            if !item.formed && names[1] == *center {
                covered.pop();
            }
            list.push(item);
        }
    }
    for (set, center, element, direction) in THREE_MEETINGS.iter() {
        if let Some(item) = full_gathering(cycles, &names, set, center, element, Some(direction), favor) {
            list.push(item);
        }
    }
    let storage = ["辰", "戌", "丑", "未"];
    if storage.iter().all(|item| names.iter().any(|name| name == item)) {
        let (verdict, verdict_label) = verdict_for_element(favor("土"), "土", "會");
        list.push(LessonBranchGathering {
            id: "branch-gathering-earth".to_string(),
            kind: "earth_full".to_string(),
            kind_label: "辰戌丑未全（土方土局）".to_string(),
            branches: "辰戌丑未".to_string(),
            positions: (0..4).map(|index| label(cycles, index)).collect(),
            element: "土".to_string(),
            formed: true,
            force_label: "四庫俱全".to_string(),
            verdict: favor_code(verdict).to_string(),
            verdict_label,
            reason: "四柱地支辰戌丑未全，可論為土方、土局，無須執著（講義第 34 頁）。".to_string(),
            notes: vec!["講義三會例六：辰戌、丑未相沖雜氣盡除，格成稼穡。".to_string()],
            source_date: SOURCE_TEXTBOOK.to_string(),
        });
    }
    list.extend(half_gatherings(cycles, &names, favor, &covered));
    list
}

/// 依本氣讀一組刑害：同類 → 加強；相生 → 加強受生者；相剋 → 削減受剋者。
fn read_relation(elements: &[Element]) -> (String, Vec<(Element, bool)>) {
    // (element, strengthened?)
    let mut effects: Vec<(Element, bool)> = Vec::new();
    let mut parts: Vec<String> = Vec::new();
    let unique = {
        let mut list: Vec<Element> = Vec::new();
        for element in elements {
            if !list.contains(element) {
                list.push(element.clone());
            }
        }
        list
    };
    if unique.len() == 1 {
        let element = unique[0].clone();
        parts.push(format!("同屬{}，加強{}力", element_name(&element), element_name(&element)));
        effects.push((element, true));
        return (parts.join("、"), effects);
    }
    // 依排列順序兩兩相讀（寅巳申讀作「木生火、火剋金」，與講義第五節例二相同）。
    for pair in unique.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        if from.get_reinforce() == *to {
            parts.push(format!("{}生{}", element_name(from), element_name(to)));
            effects.push((to.clone(), true));
        } else if to.get_reinforce() == *from {
            parts.push(format!("{}生{}", element_name(to), element_name(from)));
            effects.push((from.clone(), true));
        } else if from.get_restrain() == *to {
            parts.push(format!("{}剋{}", element_name(from), element_name(to)));
            effects.push((to.clone(), false));
        } else if to.get_restrain() == *from {
            parts.push(format!("{}剋{}", element_name(to), element_name(from)));
            effects.push((from.clone(), false));
        }
    }
    (parts.join("、"), effects)
}

fn punishment_entry(
    cycles: &[SixtyCycle; 4],
    id: String,
    kind: &str,
    kind_label: String,
    members: &[usize],
    favor: &dyn Fn(&str) -> Favor,
) -> LessonBranchPunishment {
    let mut seen: Vec<String> = Vec::new();
    let elements = members
        .iter()
        .filter(|index| {
            let name = cycles[**index].get_earth_branch().get_name();
            if seen.contains(&name) {
                false
            } else {
                seen.push(name);
                true
            }
        })
        .map(|index| main_element(cycles, *index))
        .collect::<Vec<_>>();
    let (relation, effects) = read_relation(&elements);
    let favors = effects
        .iter()
        .map(|(element, strengthened)| {
            let base = favor(&element_name(element));
            if *strengthened {
                base
            } else {
                invert(base)
            }
        })
        .collect::<Vec<_>>();
    let verdict = combine(&favors);
    let detail = effects
        .iter()
        .map(|(element, strengthened)| {
            format!(
                "{}{}（{}）",
                if *strengthened { "加強" } else { "削減" },
                element_name(element),
                favor_label(favor(&element_name(element)))
            )
        })
        .collect::<Vec<_>>()
        .join("、");
    let branches = members
        .iter()
        .map(|index| cycles[*index].get_earth_branch().get_name())
        .collect::<Vec<_>>()
        .join("");
    LessonBranchPunishment {
        id,
        kind: kind.to_string(),
        kind_label,
        branches,
        positions: members.iter().map(|index| label(cycles, *index)).collect(),
        relation: relation.clone(),
        verdict: favor_code(verdict).to_string(),
        verdict_label: match verdict {
            Favor::Good => "依生剋論，對日主有利".to_string(),
            Favor::Bad => "依生剋論，對日主不利".to_string(),
            Favor::Mixed => "依生剋論，利弊互見".to_string(),
            Favor::Neutral => "依生剋論，未列喜忌".to_string(),
        },
        reason: format!("以本氣生剋論：{}；{}。", relation, detail),
        notes: vec![
            "講義第五節：刑害的傳統論法近乎神煞，不足憑；改以五行本氣藏氣論生剋制化，配合喜忌與十神類化。".to_string(),
        ],
        source_date: SOURCE_TEXTBOOK.to_string(),
    }
}

pub(super) fn classify_punishments(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> Vec<LessonBranchPunishment> {
    let names = names(cycles);
    let mut list = Vec::new();
    let clash_or_six = |a: &str, b: &str| {
        matches!(
            (a, b),
            ("寅", "申") | ("申", "寅") | ("丑", "未") | ("未", "丑") | ("巳", "申") | ("申", "巳")
        )
    };
    for group in PUNISHMENT_GROUPS {
        let members = (0..4)
            .filter(|index| group.contains(&names[*index].as_str()))
            .collect::<Vec<_>>();
        let distinct = group
            .iter()
            .filter(|item| names.iter().any(|name| name == *item))
            .count();
        if distinct < 2 {
            continue;
        }
        // 只有兩字而這兩字本身是沖或六合（寅申、丑未、巳申），已在沖合列出，不重複列刑。
        if distinct == 2 && group.len() == 3 {
            let present = group
                .iter()
                .filter(|item| names.iter().any(|name| name == *item))
                .collect::<Vec<_>>();
            if clash_or_six(present[0], present[1]) {
                continue;
            }
        }
        // 依刑的排列順序（寅→巳→申、丑→戌→未、子→卯）讀生剋。
        let mut members = members;
        members.sort_by_key(|index| group.iter().position(|item| *item == names[*index]).unwrap_or(0));
        let full = distinct == group.len();
        let text = group.join("");
        let kind_label = if full || group.len() == 2 {
            format!("{}相刑", text)
        } else {
            {
                let mut present = members.iter().map(|index| names[*index].clone()).collect::<Vec<_>>();
                present.dedup();
                let missing = group
                    .iter()
                    .filter(|item| !names.iter().any(|name| name == *item))
                    .copied()
                    .collect::<Vec<_>>()
                    .join("");
                format!("{}相刑（{}三刑缺{}）", present.join(""), text, missing)
            }
        };
        let harm = members.len() == 2
            && HARMS.iter().any(|(a, b)| {
                (names[members[0]] == *a && names[members[1]] == *b)
                    || (names[members[0]] == *b && names[members[1]] == *a)
            });
        let kind_label = if harm { format!("{}（兼六害）", kind_label) } else { kind_label };
        list.push(punishment_entry(
            cycles,
            format!("branch-punishment-{}", text),
            "punishment",
            kind_label,
            &members,
            favor,
        ));
    }
    for name in SELF_PUNISHMENTS {
        let members = (0..4).filter(|index| names[*index] == name).collect::<Vec<_>>();
        if members.len() >= 2 {
            list.push(punishment_entry(
                cycles,
                format!("branch-self-punishment-{}", name),
                "self_punishment",
                format!("{}{}自刑", name, name),
                &members,
                favor,
            ));
        }
    }
    for (a, b) in HARMS {
        // 寅巳已在三刑列出。
        if (a, b) == ("寅", "巳") && list.iter().any(|item| item.kind == "punishment" && item.branches.contains('寅') && item.branches.contains('巳')) {
            continue;
        }
        for left in 0..4 {
            for right in left + 1..4 {
                if (names[left] == a && names[right] == b) || (names[left] == b && names[right] == a) {
                    list.push(punishment_entry(
                        cycles,
                        format!("branch-harm-{}-{}", left + 1, right + 1),
                        "harm",
                        format!("{}{}相害", names[left], names[right]),
                        &[left, right],
                        favor,
                    ));
                }
            }
        }
    }
    list
}
