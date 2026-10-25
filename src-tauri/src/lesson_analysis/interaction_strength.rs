//! 作用變化後的強弱複核（講義第二十六章「天干生剋洩合化與地支會合刑沖作用變化後」）。
//!
//! 講義第 26 章 45 例的強弱，常是「作用變化後」才下的結論（例：丁壬合化木 → 戊土轉弱；
//! 寅卯辰三會木方 → 辛金失時寡勢；子辰半合水局 → 壬水轉強）。課堂三步法的得勢只按本氣點數，
//! 所以另外做一次複核，只做提醒，不改課堂判定，也不改量化分數。
//!
//! 計算方式：
//! - 天干合而能化（書明說「丙化壬、辛化癸」）：化掉的天干改以化神五行歸黨。
//! - 三合、三會成局、半合（含中神）：地支不論化（書明說），所以不改成員的黨別，
//!   只給局神所屬的一黨加一（推論：「增其合力」的工程權重）。
//! - 得令、得地不變；用同一張課堂判定表重分類。

use super::*;

const SOURCE: &str = "講義第26章";

pub(super) fn build_interaction_strength(
    natal_cycles: &[SixtyCycle; 4],
    strength: &LessonStrengthAnalysis,
    stem_interactions: &LessonStemInteractionAnalysis,
    branch_interactions: &LessonBranchInteractionAnalysis,
) -> LessonInteractionStrengthReview {
    let day_element = natal_cycles[2].get_heaven_stem().get_element();
    let is_same = |element: &Element| matches!(family_for_element(&day_element, element), "印" | "比劫");
    let mut same = strength.same_party_count as i32;
    let mut opposing = strength.opposing_party_count as i32;
    let mut changes = Vec::new();

    for combo in &stem_interactions.combinations {
        if combo.kind != "transformed" || combo.scope != "日干以外之合" {
            continue;
        }
        let target = Element::from_name(&combo.element);
        for index in [0usize, 1] {
            let stem = natal_cycles[index].get_heaven_stem();
            let before = stem.get_element();
            if before == target || !combo.stems.contains(stem.get_name().as_str()) {
                continue;
            }
            let (from_same, to_same) = (is_same(&before), is_same(&target));
            if from_same != to_same {
                if from_same {
                    same -= 1;
                    opposing += 1;
                } else {
                    same += 1;
                    opposing -= 1;
                }
            }
            changes.push(format!(
                "{}{}合化為{}：由{}（{}）改歸{}（{}）{}",
                STEM_LOCATION_LABELS[index],
                stem.get_name(),
                combo.element,
                if from_same { "同黨" } else { "異黨" },
                family_for_element(&day_element, &before),
                if to_same { "同黨" } else { "異黨" },
                family_for_element(&day_element, &target),
                "（書明說：合化後兩干以化神論）"
            ));
        }
    }

    for item in &branch_interactions.gatherings {
        let counts = item.formed
            && matches!(item.kind.as_str(), "three_combination" | "three_meeting" | "half_combination" | "earth_full");
        if !counts {
            continue;
        }
        let element = Element::from_name(&item.element);
        if is_same(&element) {
            same += 1;
        } else {
            opposing += 1;
        }
        changes.push(format!(
            "{}：{}屬{}（{}），該黨加一（推論權重：局不論化，成員黨別不變，只記其合力）",
            item.kind_label,
            item.element,
            if is_same(&element) { "同黨" } else { "異黨" },
            family_for_element(&day_element, &element)
        ));
    }

    let month_same = strength.month_command_same_party;
    let tie = same == opposing;
    let momentum = same > opposing || (tie && month_same);
    let same_season = strength.same_season_element;
    let rooted = strength.root_count > 0;
    let observations = usize::from(same_season) + usize::from(momentum) + usize::from(rooted);
    let tie_won_by_month = tie && month_same;
    let classification = match observations {
        3 if strength.pressure_families_surface.is_empty() => "旺極候選",
        3 => "旺候選",
        2 => "強候選",
        1 if tie_won_by_month => "強候選",
        1 => "弱候選",
        _ if PRESSURE_FAMILIES
            .iter()
            .all(|family| strength.pressure_families_surface.iter().any(|item| item == family)) =>
        {
            "衰極候選"
        }
        _ => "依據不足",
    }
    .to_string();
    let changed = classification != strength.classification;
    let conclusion = if changes.is_empty() {
        "原局沒有天干合化或地支成局，作用變化不改變得勢點數。".to_string()
    } else if changed {
        format!(
            "作用變化後同黨 {}：異黨 {}（原 {}：{}），得勢{}；課堂判定可能由「{}」變為「{}」，請人工複核。",
            same,
            opposing,
            strength.same_party_count,
            strength.opposing_party_count,
            if momentum { "✓" } else { "✗" },
            strength.classification,
            classification
        )
    } else {
        format!(
            "作用變化後同黨 {}：異黨 {}（原 {}：{}），得勢{}，課堂判定仍是「{}」。",
            same,
            opposing,
            strength.same_party_count,
            strength.opposing_party_count,
            if momentum { "✓" } else { "✗" },
            classification
        )
    };
    LessonInteractionStrengthReview {
        title: "作用變化後強弱複核".to_string(),
        conclusion,
        same_party_count: same.max(0) as usize,
        opposing_party_count: opposing.max(0) as usize,
        momentum,
        classification,
        changed,
        changes,
        limitations: vec![
            "只做提醒，不取代課堂三步法的判定，也不改量化分數。".to_string(),
            "天干合化改黨別是講義明說；地支成局「該黨加一」是為了反映「增其合力」的工程權重，講義沒有給數字。".to_string(),
            "講義說八字結構複雜多端、沒有固定公式；從格、專旺、調候（如火炎土燥、金寒水冷）不在這個複核裡。".to_string(),
        ],
        sources: vec![SOURCE.to_string()],
    }
}
