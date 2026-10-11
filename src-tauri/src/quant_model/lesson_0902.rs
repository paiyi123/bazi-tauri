//! Exact source annotations, not a general luck-ranking or strength algorithm.
//! Source audit: bazi-skills/bazi-lesson-case-analysis/references/2026-09-02-male-luck-comparison.md
use crate::bazi::{
    QuantLessonAppReference, QuantLessonCandidate, QuantLessonInference, QuantModelInteraction,
};

pub(super) fn annotations(
    chart: &str,
    reference: &QuantLessonAppReference,
    interactions: &[QuantModelInteraction],
) -> Vec<QuantLessonInference> {
    if chart != "甲寅 癸酉 壬午 癸卯" {
        return Vec::new();
    }
    let mut natal = annotation(
        "natal", "本命：正印格與取用對照",
        "0902 原例：老師判正印格、身強，用火財、喜木食傷。此為原例註解；App 量化模型及課堂三步候選各自保留。",
        &[
            "事實：壬日主，酉藏辛正印；月、時兩癸為劫財；寅卯為食傷木，午本氣丁火為正財。四支無壬癸藏干，沒有直接水根。",
            "本課派說法：金印當令、兩癸扶身，老師判身強；官藏財顯，取火財、喜木生火，土官殺列其次以防土轉生金。財顯指支本氣，並非透干。",
            "補充推論：水→木→火只是可檢查的承接方向，是否有效仍受位置、力量與合沖影響；不以字數或工具百分比認證。",
            "來源：D:/Data/20260902.txt 字元 [17306,19600)，2026/09/09 核對；搜尋『四 比 四』『觀 場 財 顯』『我 怕 土 去 生 金』。字元為 UTF-8 解碼後零起算、終點不含。",
        ],
        &[
            "本課例不套從格或專旺；用火喜木依賴老師的身強判斷。",
            "課堂三步演算法依 0902 口訣『同黨異黨四比四，得令者為旺』判強候選；這是一般得勢規則，不是只為本例特判。",
            "課堂現場回饋與原始推測須分開；不能把事後解釋當作預測通過。",
        ], reference,
    );
    natal.lesson_candidate = Some(QuantLessonCandidate {
        pattern: "正印格".into(),
        strength: "身強".into(),
        use_elements: vec!["火".into()],
    });
    // Comparison scope is the two displayed fields, never a use-god or luck agreement.
    natal.agreement_status = if reference.ge_ju == "直輸四柱分析" {
        "not_comparable"
    } else if reference.ge_ju == "正印格" && reference.strength_label == "偏強" {
        "agree"
    } else {
        "conflict"
    }
    .into();
    natal.evidence.push("比對範圍：agreementStatus 只比較 App 普通格與量化強弱；不代表用神、大運或課堂三步演算法一致。".into());
    // Reuse existing natal interaction evidence; do not create another scoring path.
    for item in interactions
        .iter()
        .filter(|item| item.r#type.contains("半合") || item.r#type.contains("沖"))
    {
        natal.evidence.push(format!(
            "App 原局互動：{}（{}）；{}",
            item.target, item.r#type, item.detail
        ));
    }
    vec![
        natal,
        annotation("wuyin", "戊寅：喜木為何未直接判好",
            "課堂比較：丁丑較戊寅平順。老師提到喜神作用轉差，但逐字稿缺少完整作用鏈，不能把戊寅普遍判差。",
            &[
                "事實：運干戊為七殺；寅藏甲食神、丙偏財、戊七殺。戊與原局兩癸有五合關係；寅與午有半合火關係。這些是配對，並非已合化。",
                "本課派說法：老師比較丁丑火土與戊寅土木，說後者喜神作用轉差；課堂結論為丁丑較平順。",
                "補充推論：寅木可洩水生火，也可制戊土；戊可制水，也可能生酉金再扶水。需比較這些相反路徑，不能只見喜木就判優。",
                "來源：D:/Data/20260902.txt 字元 [19000,21300)，2026/09/09 核對；搜尋『喜 生 變 訣 竅』『應 酬 運 會比 戊 寅 運 順利』。前一句 ASR 不清，後句『應酬運』依前文丁丑／戊寅比較作語境整理。",
            ],
            &[
                "戊癸相合不等於兩癸消失或已化火；未提供成立門檻，不能計分。",
                "原局已有寅午關係，大運寅再現不自動加倍；不能宣稱半合已變完整火局。",
                "現場工作與業外投資談話只留文件來源，不轉成職涯或財務建議。",
            ], reference),
        annotation("jimao", "己卯：沖忌印的課堂解釋",
            "課堂比較：己卯較戊寅好一些。老師明說卯酉沖使忌印受傷；這是原例的相對評價，尚非一般沖忌判吉公式。",
            &[
                "事實：己為正官，卯藏乙傷官；運支卯與原局酉相沖，與午相破，運干己與年干甲有五合關係。原局時支已是卯。",
                "本課派說法：老師以卯酉沖使忌神酉印受傷，解釋己卯較戊寅改善；『天干就土沒有什麼問題』是口述概評，未細分己官的作用。",
                "補充推論：卯木可以洩水、生午火，也可能受酉金反制或制己官；甲己相合是否牽制甲木承接火，需另查條件。這些不是老師已講出的因果鏈。",
                "來源：D:/Data/20260902.txt 字元 [21000,21500)，2026/09/09 核對；搜尋『忌 神 受傷 是可以 的』『吉 卯 印 會比 戊 寅 印 好 一些』。『吉卯印』按大運序列整理為己卯運。",
            ],
            &[
                "沖是雙向關係，卯喜神是否受損也要檢查；酉及辛藏干仍然存在。",
                "原局已有卯酉沖；大運是再引動，不作兩倍沖分數。甲己合不直接當作化土。",
                "『較好一些』不等於絕對吉；不推現實事件或收益。",
            ], reference),
        annotation("gengchen", "庚辰：保留結論與證據缺口",
            "課堂記錄：老師也稱庚辰好運，但作用鏈未交代。保留來源結論，無法據此排出庚辰與己卯的高低。",
            &[
                "事實：庚為偏印；辰藏戊七殺、乙傷官、癸劫財。辰與原局酉有六合關係；年支寅、時支卯與運支辰湊齊寅卯辰三會的支字。這不代表六合或三會已化。",
                "本課派說法：逐字稿明載『喜 卯 印 好運 庚 辰 印 也是 好運』，但這句前後沒有交代庚辰如何改善本局。",
                "補充推論：土生金、金生水可增加印比；庚金也可能制甲木。辰酉合與寅卯辰三會涉及不同方向，須有優先序與有效力量判準，不能挑一條替『好運』補理由。",
                "來源：D:/Data/20260902.txt 字元 [23900,24300)，2026/09/09 核對；搜尋『庚 辰 印 也是 好運』。僅確認老師有此評語，並未驗證評語正確。",
            ],
            &[
                "不能說辰是濕土所以一定好；後面的濕土與回剋講解屬另一個女命，不移植成本例來源。",
                "不能因六合就刪除酉印，或因三會支字齊就宣稱木局成化。",
                "尚缺合會競合、季節與位置權重，以及庚辰相對己卯的直接比較；不產生通用大運分數。",
            ], reference),
    ]
}

fn annotation(
    id: &str,
    title: &str,
    conclusion: &str,
    evidence: &[&str],
    rejected: &[&str],
    reference: &QuantLessonAppReference,
) -> QuantLessonInference {
    QuantLessonInference {
        rule_id: format!("20260902-male-{id}"),
        title: format!("2026/09/02 男命原例｜{title}"),
        status: "已比對原例・課堂註解".into(),
        source_verification: "dated_lesson_chart_and_transcript_rechecked".into(),
        algorithm_status: "candidate".into(), match_status: "matched".into(),
        output_mode: "annotation_only".into(), fixture_scope: "source_case_only".into(),
        agreement_status: "not_comparable".into(), fallback_policy: "preserve_normal_outputs".into(),
        app_reference: reference.clone(), lesson_candidate: None,
        sensitive_claims_policy: "excluded".into(), conclusion: conclusion.into(),
        evidence: evidence.iter().map(|s| s.to_string()).collect(),
        rejected_alternatives: rejected.iter().map(|s| s.to_string()).chain([
            "適用範圍：只比對本課四柱原例；大運序列照錄男命課例，未驗算生日與交運日期，不表示目前正行該運。".into(),
        ]).collect(),
    }
}
