//! 天干生剋合化的作用變化（草綠色講義第二十三章，2026-10-07、2026-10-14）。
//!
//! - 10/07：天干相生相剋看遠近（鄰近、隔一位、遙隔）、坐下通根或截腳；通關／貪生忘剋、
//!   先受剋者無力剋人、近者優先、連環剋。
//! - 10/14：天干五合分合而不合、合而不化、合來合去、爭合妒合、合而能化，再依喜忌定吉凶。
//!
//! 只做結構判斷與喜忌標示，不改量化分數，也不回頭改強弱與喜忌方案。

use super::*;
use tyme4rs::tyme::sixtycycle::EarthBranch;

const COMBINATIONS: [(&str, &str, &str); 5] = [
    ("甲", "己", "土"),
    ("乙", "庚", "金"),
    ("丙", "辛", "水"),
    ("丁", "壬", "木"),
    ("戊", "癸", "火"),
];

/// 三會、三合局：局中地支的本氣根會被局的五行帶走。
const BRANCH_FRAMES: [([&str; 3], &str); 8] = [
    (["寅", "卯", "辰"], "木"),
    (["巳", "午", "未"], "火"),
    (["申", "酉", "戌"], "金"),
    (["亥", "子", "丑"], "水"),
    (["申", "子", "辰"], "水"),
    (["亥", "卯", "未"], "木"),
    (["寅", "午", "戌"], "火"),
    (["巳", "酉", "丑"], "金"),
];

const SOURCE_RELATIONS: &str = "2026-10-07";
const SOURCE_COMBINATIONS: &str = "2026-10-14";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Favor {
    Good,
    Bad,
    Mixed,
    Neutral,
}

impl Favor {
    fn code(self) -> &'static str {
        match self {
            Favor::Good => "good",
            Favor::Bad => "bad",
            Favor::Mixed => "mixed",
            Favor::Neutral => "neutral",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Favor::Good => "喜用",
            Favor::Bad => "忌",
            Favor::Mixed => "依方案而定",
            Favor::Neutral => "未列喜忌",
        }
    }
}

/// 天干五合的化神（甲己→土……）。
pub(crate) fn combination_element(left: &str, right: &str) -> Option<&'static str> {
    COMBINATIONS
        .iter()
        .find(|(a, b, _)| (*a == left && *b == right) || (*a == right && *b == left))
        .map(|(_, _, element)| *element)
}

/// 支的本氣是化神或生化神（申酉之金可生丙辛所化之水）。
fn branch_main_supports(branch: &EarthBranch, element: &Element) -> bool {
    let main = branch.get_element();
    main == *element || main == element.get_reinforced()
}

fn branch_holds_element(branch: &EarthBranch, element: &Element) -> bool {
    branch
        .get_hide_heaven_stems()
        .iter()
        .any(|hidden| hidden.get_heaven_stem().get_element() == *element)
}

/// 合神所臨之支「化神通根乘旺」：兩支本氣都是化神或生化神，且至少一支藏有化神之根。
/// 講義例：戊癸臨巳午、甲己臨未戌、丙辛臨申酉（申藏壬水）。
fn pillars_support_transformation(left: &EarthBranch, right: &EarthBranch, element: &Element) -> bool {
    branch_main_supports(left, element)
        && branch_main_supports(right, element)
        && (branch_holds_element(left, element) || branch_holds_element(right, element))
}

fn layer_weight(layer: HideHeavenStemType) -> f64 {
    match layer {
        HideHeavenStemType::MAIN => 1.0,
        HideHeavenStemType::MIDDLE => 0.6,
        HideHeavenStemType::RESIDUAL => 0.3,
    }
}

fn branch_root_weight(branch: &EarthBranch, element: &Element) -> f64 {
    branch
        .get_hide_heaven_stems()
        .into_iter()
        .filter(|hidden| hidden.get_heaven_stem().get_element() == *element)
        .map(|hidden| layer_weight(hidden.get_type()))
        .sum()
}

fn root_weight(cycles: &[SixtyCycle; 4], element: &Element) -> f64 {
    cycles
        .iter()
        .map(|cycle| branch_root_weight(&cycle.get_earth_branch(), element))
        .sum()
}

/// 坐支若已入三會／三合局且局的五行不同，根被帶走（例：戊坐辰，但寅卯辰會木）。
fn frame_taking_seat(cycles: &[SixtyCycle; 4], index: usize, element: &Element) -> Option<String> {
    let names = cycles
        .iter()
        .map(|cycle| cycle.get_earth_branch().get_name())
        .collect::<Vec<_>>();
    let seat = &names[index];
    BRANCH_FRAMES
        .iter()
        .find(|(members, frame_element)| {
            members.contains(&seat.as_str())
                && members.iter().all(|member| names.iter().any(|name| name == member))
                && *frame_element != element_name(element)
        })
        .map(|(members, frame_element)| format!("{}成{}局", members.join(""), frame_element))
}

/// 坐下通根（坐旺）：自坐之支藏有同五行，且沒有被會局帶走。
fn seat_rooted(cycles: &[SixtyCycle; 4], index: usize) -> bool {
    let element = cycles[index].get_heaven_stem().get_element();
    branch_root_weight(&cycles[index].get_earth_branch(), &element) > 0.0
        && frame_taking_seat(cycles, index, &element).is_none()
}

/// 比較牽絆方向用的粗略力量：地支根氣（本氣1、中氣0.6、餘氣0.3）加天干同黨（每個0.4）。
fn stem_power(cycles: &[SixtyCycle; 4], index: usize, partner: usize) -> f64 {
    let element = cycles[index].get_heaven_stem().get_element();
    let peers = (0..4)
        .filter(|other| *other != index && *other != partner)
        .filter(|other| cycles[*other].get_heaven_stem().get_element() == element)
        .count();
    root_weight(cycles, &element) + 0.4 * peers as f64
}

fn stem_label(cycles: &[SixtyCycle; 4], index: usize) -> String {
    let day = cycles[2].get_heaven_stem();
    let stem = cycles[index].get_heaven_stem();
    let ten_god = if index == 2 {
        "日元".to_string()
    } else {
        ten_god_name(&day, &stem)
    };
    format!("{}{}（{}）", STEM_LOCATION_LABELS[index], stem.get_name(), ten_god)
}

fn short_label(cycles: &[SixtyCycle; 4], index: usize) -> String {
    format!(
        "{}{}",
        STEM_LOCATION_LABELS[index],
        cycles[index].get_heaven_stem().get_name()
    )
}

/// 課堂喜忌：任一方案列為用神或喜神、且沒有方案列為忌 → 喜用；反之為忌；兩邊都有 → 依方案而定。
pub(crate) fn element_favor(preferences: &LessonPreferenceAnalysis, element: &str) -> Favor {
    let mut good = false;
    let mut bad = false;
    for plan in &preferences.plans {
        good |= plan
            .use_directions
            .iter()
            .chain(plan.favorable_directions.iter())
            .any(|direction| direction.element == element);
        bad |= plan
            .unfavorable_directions
            .iter()
            .any(|direction| direction.element == element);
    }
    match (good, bad) {
        (true, true) => Favor::Mixed,
        (true, false) => Favor::Good,
        (false, true) => Favor::Bad,
        (false, false) => Favor::Neutral,
    }
}

fn favor_of(cycles: &[SixtyCycle; 4], index: usize, favor: &dyn Fn(&str) -> Favor) -> Favor {
    favor(&element_name(&cycles[index].get_heaven_stem().get_element()))
}

fn combine_favors(favors: &[Favor]) -> Favor {
    let decided = favors
        .iter()
        .filter(|item| matches!(item, Favor::Good | Favor::Bad))
        .collect::<BTreeSet<_>>();
    if favors.contains(&Favor::Mixed) || decided.len() > 1 {
        Favor::Mixed
    } else {
        decided.into_iter().next().copied().unwrap_or(Favor::Neutral)
    }
}

impl PartialOrd for Favor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Favor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

/// 被牽絆的一方是喜用 → 減吉；是忌 → 減凶。
fn bound_effect(favor: Favor) -> (Favor, &'static str) {
    match favor {
        Favor::Good => (Favor::Bad, "減吉"),
        Favor::Bad => (Favor::Good, "減凶"),
        Favor::Mixed => (Favor::Mixed, "依方案而定"),
        Favor::Neutral => (Favor::Neutral, "未列喜忌"),
    }
}

/// 合來的一方是喜用 → 增吉；是忌 → 增凶。
fn attracted_effect(favor: Favor) -> (Favor, &'static str) {
    match favor {
        Favor::Good => (Favor::Good, "增吉"),
        Favor::Bad => (Favor::Bad, "增凶"),
        Favor::Mixed => (Favor::Mixed, "依方案而定"),
        Favor::Neutral => (Favor::Neutral, "未列喜忌"),
    }
}

struct ComboDraft {
    indices: Vec<usize>,
    stems: String,
    element: &'static str,
    scope: &'static str,
    kind: &'static str,
    kind_label: &'static str,
    direction: Option<&'static str>,
    bound: Vec<String>,
    verdict: Favor,
    verdict_label: String,
    reason: String,
    notes: Vec<String>,
    needs_confirmation: bool,
}

impl ComboDraft {
    fn into_dto(self, cycles: &[SixtyCycle; 4]) -> LessonStemCombination {
        LessonStemCombination {
            id: format!(
                "stem-combination-{}",
                self.indices
                    .iter()
                    .map(|index| (index + 1).to_string())
                    .collect::<Vec<_>>()
                    .join("-")
            ),
            stems: self.stems,
            positions: self
                .indices
                .iter()
                .map(|index| stem_label(cycles, *index))
                .collect(),
            element: self.element.to_string(),
            scope: self.scope.to_string(),
            kind: self.kind.to_string(),
            kind_label: self.kind_label.to_string(),
            direction: self.direction.map(str::to_string),
            bound_stems: self.bound,
            verdict: self.verdict.code().to_string(),
            verdict_label: self.verdict_label,
            reason: self.reason,
            notes: self.notes,
            needs_confirmation: self.needs_confirmation,
            source_date: SOURCE_COMBINATIONS.to_string(),
        }
    }
}

fn partner_of(stem: &str) -> Option<&'static str> {
    COMBINATIONS.iter().find_map(|(a, b, _)| {
        if *a == stem {
            Some(*b)
        } else if *b == stem {
            Some(*a)
        } else {
            None
        }
    })
}

/// 爭合只限講義列的排列：日干在中，月時兩干同為合神（年干也是亦同）；或年干與日干同、月干為合神。
fn contested_combination(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> Option<ComboDraft> {
    let names = cycles
        .iter()
        .map(|cycle| cycle.get_heaven_stem().get_name())
        .collect::<Vec<_>>();
    let day = names[2].as_str();
    let partner = partner_of(day)?;
    let element = combination_element(day, partner)?;
    let day_stem = cycles[2].get_heaven_stem();
    let target_family = if day_stem.get_yin_yang() == tyme4rs::tyme::enums::YinYang::YANG {
        "財"
    } else {
        "官"
    };
    if names[1] == partner && names[3] == partner {
        // 合神（財或官）兩三個去爭合日主：同合來，看合神喜忌。
        let mut indices = if names[0] == partner {
            vec![0, 1, 2, 3]
        } else {
            vec![1, 2, 3]
        };
        indices.sort();
        let count = indices.len() - 1;
        let partner_favor = favor_of(cycles, 1, favor);
        let (verdict, effect) = attracted_effect(partner_favor);
        let partner_name = element_name(&cycles[1].get_heaven_stem().get_element());
        let effect_text = match partner_favor {
            Favor::Good => format!("{}為喜用，日主逢{}約束反而有助（講義例四：三丙爭合辛）", target_family, target_family),
            Favor::Bad => format!("{}為忌，日主貪戀{}、三心兩意，終非美事（講義例三：兩壬爭合丁）", target_family, target_family),
            _ => format!("{}喜忌未定", target_family),
        };
        return Some(ComboDraft {
            stems: format!("{}{}", partner, day),
            indices,
            element,
            scope: "日干本身之合",
            kind: "contested",
            kind_label: "爭合妒合",
            direction: Some("合來"),
            bound: vec![stem_label(cycles, 2)],
            verdict,
            verdict_label: format!("爭合{}", effect),
            reason: format!(
                "{}個{}（{}，{}）夾日干{}爭合；爭合不論化。{}。",
                count,
                partner,
                target_family,
                partner_name,
                day,
                effect_text
            ),
            notes: vec!["爭合只限日干在中、兩邊（或年月時三干）同為合神，或年干與日干同、月干為合神的排列。".to_string()],
            needs_confirmation: false,
        });
    }
    if names[1] == partner && names[0] == day {
        // 年干與日干同去爭合月干：被爭的是日主的財（陽干）或官（陰干）。
        let partner_favor = favor_of(cycles, 1, favor);
        let (verdict, effect) = match partner_favor {
            Favor::Good => (Favor::Bad, format!("損{}", target_family)),
            Favor::Bad => (
                Favor::Good,
                if target_family == "財" {
                    "幫身分財".to_string()
                } else {
                    "幫身敵官".to_string()
                },
            ),
            Favor::Mixed => (Favor::Mixed, "依方案而定".to_string()),
            Favor::Neutral => (Favor::Neutral, "未列喜忌".to_string()),
        };
        return Some(ComboDraft {
            stems: format!("{}{}", day, partner),
            indices: vec![0, 1, 2],
            element,
            scope: "日干以外之合",
            kind: "contested",
            kind_label: "爭合妒合",
            direction: None,
            bound: vec![stem_label(cycles, 1)],
            verdict,
            verdict_label: format!("爭合{}", effect),
            reason: format!(
                "年干、日干兩{}爭合月干{}（{}，{}）；爭合不論化。{}為{} → {}。",
                day,
                partner,
                target_family,
                element_name(&cycles[1].get_heaven_stem().get_element()),
                target_family,
                partner_favor.label(),
                effect
            ),
            notes: vec!["爭合只限日干在中、兩邊（或年月時三干）同為合神，或年干與日干同、月干為合神的排列。".to_string()],
            needs_confirmation: false,
        });
    }
    None
}

/// 合神被鄰干剋，且自坐無根，自顧不暇不敢去合。
fn blocked_by_control(cycles: &[SixtyCycle; 4], left: usize, right: usize) -> Option<String> {
    for (target, partner) in [(left, right), (right, left)] {
        let neighbours = [target.checked_sub(1), Some(target + 1)];
        for neighbour in neighbours.into_iter().flatten() {
            if neighbour > 3 || neighbour == partner {
                continue;
            }
            let attacker = cycles[neighbour].get_heaven_stem().get_element();
            let element = cycles[target].get_heaven_stem().get_element();
            if attacker.get_restrain() != element || root_weight(cycles, &attacker) <= 0.0 {
                continue;
            }
            if seat_rooted(cycles, target) {
                continue;
            }
            let seat = cycles[target].get_earth_branch().get_name();
            let seat_note = match frame_taking_seat(cycles, target, &element) {
                Some(frame) => format!("自坐{}但{}，根被帶走", seat, frame),
                None => format!("自坐{}無根", seat),
            };
            return Some(format!(
                "{}被鄰近的{}剋，{}，自顧不暇，不敢去合{}",
                short_label(cycles, target),
                short_label(cycles, neighbour),
                seat_note,
                short_label(cycles, partner)
            ));
        }
    }
    None
}

/// 日干本身之合能化：其餘六字（另外兩干、四支本氣）都是化神或生化神（10/07、10/14 例五、例六）。
fn day_transformation(cycles: &[SixtyCycle; 4], pair: (usize, usize), element: &Element) -> bool {
    let generator = element.get_reinforced();
    let stems_ok = (0..4)
        .filter(|index| *index != pair.0 && *index != pair.1)
        .all(|index| {
            let current = cycles[index].get_heaven_stem().get_element();
            current == *element || current == generator
        });
    let branches_ok = cycles.iter().all(|cycle| {
        let current = cycle.get_earth_branch().get_element();
        current == *element || current == generator
    });
    stems_ok && branches_ok
}

fn pair_combination(
    cycles: &[SixtyCycle; 4],
    left: usize,
    right: usize,
    element: &'static str,
    favor: &dyn Fn(&str) -> Favor,
) -> ComboDraft {
    let left_name = cycles[left].get_heaven_stem().get_name();
    let right_name = cycles[right].get_heaven_stem().get_name();
    let stems = format!("{}{}", left_name, right_name);
    let element_value = Element::from_name(element);
    let includes_day = left == 2 || right == 2;
    let blank = |kind, kind_label, scope| ComboDraft {
        indices: vec![left, right],
        stems: stems.clone(),
        element,
        scope,
        kind,
        kind_label,
        direction: None,
        bound: Vec::new(),
        verdict: Favor::Neutral,
        verdict_label: String::new(),
        reason: String::new(),
        notes: Vec::new(),
        needs_confirmation: false,
    };

    // 一、合而不合：中間有間隔。
    if right - left > 1 {
        let between = (left + 1..right)
            .map(|index| short_label(cycles, index))
            .collect::<Vec<_>>()
            .join("、");
        let mut draft = blank("blocked", "合而不合", "隔位相合");
        draft.verdict_label = "不論合".to_string();
        draft.reason = format!(
            "{}與{}中間隔著{}，不能越位而合，照一般生剋論。",
            short_label(cycles, left),
            short_label(cycles, right),
            between
        );
        draft
            .notes
            .push("老師說法：坊間把隔位的天干也當成合來論，是論命不準的原因之一。".to_string());
        return draft;
    }

    let scope = if includes_day {
        "日干本身之合"
    } else {
        "日干以外之合"
    };

    // 一、合而不合：合神被剋不敢合。
    if let Some(reason) = blocked_by_control(cycles, left, right) {
        let mut draft = blank("blocked", "合而不合", scope);
        draft.verdict_label = "不論合".to_string();
        draft.reason = format!("{}，照一般生剋論。", reason);
        return draft;
    }

    let other = if left == 2 { right } else { left };
    let mut seat_note: Option<String> = None;

    // 五、合而能化。
    if includes_day {
        if day_transformation(cycles, (left, right), &element_value) {
            let day_element = cycles[2].get_heaven_stem().get_element();
            let mut draft = blank("transformed", "合而能化", scope);
            draft.verdict = Favor::Neutral;
            if day_element == element_value {
                draft.verdict_label = "化氣（類似專旺）".to_string();
                draft.reason = format!(
                    "{}合，其餘六字都是{}或生{}之神，其勢必化：{}化為{}，日主{}本身就是{}，實與專旺無異，以專旺論喜忌（講義例五）。",
                    stems, element, element, short_label(cycles, other), element, cycles[2].get_heaven_stem().get_name(), element
                );
            } else {
                draft.verdict_label = "從象（化氣）".to_string();
                draft.reason = format!(
                    "{}合，其餘六字都是{}或生{}之神：日主不能論化，但必順{}之氣勢，合成從象，以從格論喜忌（講義例六）。",
                    stems, element, element, element
                );
            }
            draft.notes.push(
                "老師說法：統稱化氣格也無不可，喜忌論法都一樣（喜化神與生化神）；這時普通格的扶抑喜忌要改看特殊格。"
                    .to_string(),
            );
            return draft;
        }
    } else {
        let left_branch = cycles[left].get_earth_branch();
        let right_branch = cycles[right].get_earth_branch();
        let hour_branch = cycles[3].get_earth_branch().get_name();
        let seats_ok = pillars_support_transformation(&left_branch, &right_branch, &element_value);
        if seats_ok && hour_branch == "辰" {
            let result = favor(element);
            let (verdict, effect) = match result {
                Favor::Good => (Favor::Good, "合化為喜，吉"),
                Favor::Bad => (Favor::Bad, "合化為忌，凶"),
                Favor::Mixed => (Favor::Mixed, "依方案而定"),
                Favor::Neutral => (Favor::Neutral, "未列喜忌"),
            };
            let changed = [left, right]
                .into_iter()
                .filter(|index| cycles[*index].get_heaven_stem().get_element() != element_value)
                .map(|index| format!("{}化為{}", short_label(cycles, index), element))
                .collect::<Vec<_>>()
                .join("、");
            let mut draft = blank("transformed", "合而能化", scope);
            draft.verdict = verdict;
            draft.verdict_label = effect.to_string();
            draft.reason = format!(
                "年月{}合，生在辰時，所坐{}{}為化神通根乘旺 → {}；兩干都以{}論，{}為{}。",
                stems,
                left_branch.get_name(),
                right_branch.get_name(),
                changed,
                element,
                element,
                result.label()
            );
            draft.notes.push(
                "10/21 老師：日干以外之合化不化，第一看是否生在辰時（引至歸時見辰而化），第二看所坐地支化神通根乘旺。"
                    .to_string(),
            );
            draft.notes.push("合化後的五行沒有回頭重算強弱與喜忌方案。".to_string());
            return draft;
        }
        if seats_ok {
            seat_note = Some(format!(
                "所坐{}{}雖為化神通根乘旺，但時支是{}不是辰；10/21 老師列「生在辰時」為年月合化的第一個條件，所以不化。",
                left_branch.get_name(),
                right_branch.get_name(),
                hour_branch
            ));
        }
    }

    // 三、四：合而不化（剋合），分合來、合去。
    let mut draft = blank("bound", "合而不化", scope);
    if let Some(note) = seat_note {
        draft.notes.push(note);
    }
    draft.notes.push("合而不化以剋合論：合中仍帶剋，只是陰陽相見剋力較小（10/07）。".to_string());
    if includes_day {
        let other_favor = favor_of(cycles, other, favor);
        let (verdict, effect) = attracted_effect(other_favor);
        draft.direction = Some("合來");
        draft.bound = vec![stem_label(cycles, other)];
        draft.verdict = verdict;
        draft.verdict_label = format!("合來{}", effect);
        draft.reason = format!(
            "{}合，化神之力不足而不化；日干本身之合為合來，{}心向日主。{}為{} → 合來{}。",
            stems,
            stem_label(cycles, other),
            element_name(&cycles[other].get_heaven_stem().get_element()),
            other_favor.label(),
            effect
        );
        draft
            .notes
            .push("日干本身之合不論羈絆、不為合去（三命通會、徐樂吾）。".to_string());
        return draft;
    }

    let left_power = stem_power(cycles, left, right);
    let right_power = stem_power(cycles, right, left);
    draft.direction = Some("合去");
    let power_note = format!(
        "力量：{} {:.1}、{} {:.1}（根氣本氣1、中氣0.6、餘氣0.3，天干同黨各0.4）",
        short_label(cycles, left),
        left_power,
        short_label(cycles, right),
        right_power
    );
    if (left_power - right_power).abs() < 0.3 {
        let favors = [left, right].map(|index| bound_effect(favor_of(cycles, index, favor)).0);
        draft.bound = vec![stem_label(cycles, left), stem_label(cycles, right)];
        draft.verdict = combine_favors(&favors);
        let effects = [left, right]
            .into_iter()
            .map(|index| {
                format!(
                    "{}為{}（{}）",
                    short_label(cycles, index),
                    favor_of(cycles, index, favor).label(),
                    bound_effect(favor_of(cycles, index, favor)).1
                )
            })
            .collect::<Vec<_>>()
            .join("、");
        draft.verdict_label = format!("合去，{}", match draft.verdict {
            Favor::Good => "減凶",
            Favor::Bad => "減吉",
            _ => "吉凶互見",
        });
        draft.reason = format!(
            "年月{}合而不化，兩干力量相當，互相牽絆：{}；喜忌力量仍在，只是減弱。",
            stems, effects
        );
    } else {
        let (strong, weak) = if left_power > right_power {
            (left, right)
        } else {
            (right, left)
        };
        let weak_favor = favor_of(cycles, weak, favor);
        let (verdict, effect) = bound_effect(weak_favor);
        draft.bound = vec![stem_label(cycles, weak)];
        draft.verdict = verdict;
        draft.verdict_label = format!("合去{}", effect);
        draft.reason = format!(
            "年月{}合而不化，力大者絆住力小者：{}絆住{}，{}不為日主所用。{}為{} → {}。",
            stems,
            short_label(cycles, strong),
            stem_label(cycles, weak),
            short_label(cycles, weak),
            element_name(&cycles[weak].get_heaven_stem().get_element()),
            weak_favor.label(),
            effect
        );
    }
    draft.notes.push(power_note);
    draft
}

pub(super) fn classify_combinations(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> Vec<LessonStemCombination> {
    let mut drafts = Vec::new();
    let contested = contested_combination(cycles, favor);
    let consumed = |left: usize, right: usize| {
        contested
            .as_ref()
            .map(|draft| draft.indices.contains(&left) && draft.indices.contains(&right) && (left == 2 || right == 2 || (left, right) == (0, 1)))
            .unwrap_or(false)
    };
    for left in 0..4 {
        for right in left + 1..4 {
            let left_name = cycles[left].get_heaven_stem().get_name();
            let right_name = cycles[right].get_heaven_stem().get_name();
            let Some(element) = combination_element(&left_name, &right_name) else {
                continue;
            };
            if consumed(left, right) {
                continue;
            }
            drafts.push(pair_combination(cycles, left, right, element, favor));
        }
    }
    if let Some(draft) = contested {
        drafts.insert(0, draft);
    }
    drafts.into_iter().map(|draft| draft.into_dto(cycles)).collect()
}

fn distance_label(distance: usize) -> &'static str {
    match distance {
        1 => "鄰近",
        2 => "隔一位",
        _ => "遙隔",
    }
}

fn generates(from: &Element, to: &Element) -> bool {
    from.get_reinforce() == *to
}

fn controls(from: &Element, to: &Element) -> bool {
    from.get_restrain() == *to
}

pub(super) fn classify_relations(cycles: &[SixtyCycle; 4]) -> Vec<LessonStemRelation> {
    let elements = cycles
        .iter()
        .map(|cycle| cycle.get_heaven_stem().get_element())
        .collect::<Vec<_>>();
    let names = cycles
        .iter()
        .map(|cycle| cycle.get_heaven_stem().get_name())
        .collect::<Vec<_>>();
    let mut relations = Vec::new();
    for from in 0..4 {
        for to in 0..4 {
            if from == to || combination_element(&names[from], &names[to]).is_some() {
                continue;
            }
            let (relation, is_control) = if generates(&elements[from], &elements[to]) {
                ("生", false)
            } else if controls(&elements[from], &elements[to]) {
                ("剋", true)
            } else {
                continue;
            };
            let distance = from.abs_diff(to);
            let (low, high) = (from.min(to), from.max(to));
            let from_label = short_label(cycles, from);
            let to_label = short_label(cycles, to);
            let rooted = seat_rooted(cycles, from);
            let (status, status_label, reason) = if !is_control {
                let power = match distance {
                    1 => "相鄰貼近，生力大",
                    2 => "隔一位，生力較小",
                    _ => "遙隔，生力更小",
                };
                let seat = if rooted {
                    "，坐旺通根生力足"
                } else {
                    "，但坐下無根（坐剋虛浮），生力打折"
                };
                ("active", "論生", format!("{}{}。", power, seat))
            } else if let Some(middle) = (low + 1..high)
                .find(|middle| generates(&elements[from], &elements[*middle]) && generates(&elements[*middle], &elements[to]))
            {
                (
                    "ignored",
                    "不論剋",
                    format!(
                        "通關：{}生{}、{}生{}，論轉生不論剋（貪生忘剋）。",
                        from_label,
                        short_label(cycles, middle),
                        short_label(cycles, middle),
                        to_label
                    ),
                )
            } else if let Some(attacker) = [from.checked_sub(1), Some(from + 1)]
                .into_iter()
                .flatten()
                .find(|index| *index <= 3 && *index != to && controls(&elements[*index], &elements[from]) && combination_element(&names[*index], &names[from]).is_none())
            {
                let attacker_label = short_label(cycles, attacker);
                if distance > 1 {
                    (
                        "ignored",
                        "不論剋",
                        format!("先受剋者無力剋人：{}先受{}剋，不論{}剋{}。", from_label, attacker_label, from_label, to_label),
                    )
                } else if rooted {
                    (
                        "active",
                        "論剋",
                        format!("連環剋：{}雖受{}剋，但坐旺通根，仍有餘力剋{}。", from_label, attacker_label, to_label),
                    )
                } else {
                    (
                        "ignored",
                        "不論剋",
                        format!("連環剋：{}受{}剋，自顧不暇，不去剋{}。", from_label, attacker_label, to_label),
                    )
                }
            } else {
                let from_power = root_weight(cycles, &elements[from]);
                let to_power = root_weight(cycles, &elements[to]);
                if !rooted && to_power >= 2.0 && from_power * 2.0 < to_power {
                    (
                        "weak",
                        "剋力有限",
                        format!(
                            "{}坐下無根、根氣{:.1}，{}結黨有根（根氣{:.1}），剋者反而無能為力（如土旺木折、木旺金缺）。",
                            from_label, from_power, to_label, to_power
                        ),
                    )
                } else {
                    let power = match distance {
                        1 => "相鄰貼近，剋力較大",
                        2 => "隔一位，剋力較輕",
                        _ => "遙隔，剋力更輕",
                    };
                    let mut reason = format!("{}。", power);
                    if distance == 1 {
                        if let Some(far) = (0..4).find(|index| {
                            (*index < low || *index > high)
                                && generates(&elements[from], &elements[*index])
                                && generates(&elements[*index], &elements[to])
                        }) {
                            reason.push_str(&format!(
                                "{}雖可通關，但遠隔，{}、{}相鄰先剋（近者優先）。",
                                short_label(cycles, far),
                                from_label,
                                to_label
                            ));
                        }
                    }
                    ("active", "論剋", reason)
                }
            };
            relations.push(LessonStemRelation {
                id: format!("stem-relation-{}-{}", from + 1, to + 1),
                from: stem_label(cycles, from),
                to: stem_label(cycles, to),
                relation: relation.to_string(),
                distance: distance_label(distance).to_string(),
                status: status.to_string(),
                status_label: status_label.to_string(),
                reason,
                source_date: SOURCE_RELATIONS.to_string(),
            });
        }
    }
    relations.sort_by_key(|item| item.id.clone());
    relations
}

pub(super) fn build_stem_interactions(
    natal_cycles: &[SixtyCycle; 4],
    preferences: &LessonPreferenceAnalysis,
) -> LessonStemInteractionAnalysis {
    let favor = |element: &str| element_favor(preferences, element);
    let combinations = classify_combinations(natal_cycles, &favor);
    let relations = classify_relations(natal_cycles);
    let conclusion = if combinations.is_empty() {
        format!(
            "天干沒有五合；生剋 {} 組（其中不論 {} 組）。",
            relations.len(),
            relations.iter().filter(|item| item.status == "ignored").count()
        )
    } else {
        let parts = combinations
            .iter()
            .map(|item| {
                format!(
                    "{}{}{}（{}）",
                    item.stems,
                    item.kind_label,
                    item.direction
                        .as_ref()
                        .map(|direction| format!("・{}", direction))
                        .unwrap_or_default(),
                    item.verdict_label
                )
            })
            .collect::<Vec<_>>()
            .join("；");
        format!(
            "天干五合：{}。生剋 {} 組（其中不論 {} 組）。",
            parts,
            relations.len(),
            relations.iter().filter(|item| item.status == "ignored").count()
        )
    };
    let status = if combinations.iter().any(|item| item.needs_confirmation) || preferences.plans.len() > 1 {
        "conditional"
    } else {
        "candidate"
    };
    LessonStemInteractionAnalysis {
        title: "天干生剋合化".to_string(),
        conclusion,
        status: status.to_string(),
        combinations,
        relations,
        limitations: vec![
            "吉凶依課堂三步法的喜忌方案；多方案時，元素只有在沒有方案相反時才算喜或忌。".to_string(),
            "牽絆方向、虛浮與否用根氣粗估，不是老師的量化公式。".to_string(),
            "合化後的五行沒有回頭重算強弱；量化五行力分數不受影響。".to_string(),
        ],
        sources: vec![SOURCE_RELATIONS.to_string(), SOURCE_COMBINATIONS.to_string()],
    }
}
