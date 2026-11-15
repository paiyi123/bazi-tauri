// 解盤分區的清單：解盤頁（AnalysisTopics）和電腦版左欄目錄（TopicSideNav）共用。

export type TopicKey = "steps" | "learn" | "strength" | "pattern" | "interaction" | "luck" | "family" | "health" | "career" | "reference";

export const TOPICS: { key: TopicKey; short: string; title: string; desc: string }[] = [
  { key: "steps", short: "解盤步驟", title: "解盤 10 步", desc: "照老師的順序一步一步看這張盤，每一步都寫出為什麼；點「看詳細」跳到該分區。" },
  { key: "luck", short: "大運流年", title: "大運流年", desc: "大運、流年、流月與各書的運歲標籤；看一年的步驟、限運年齡。" },
  { key: "strength", short: "強弱用神", title: "強弱・用神・喜忌", desc: "日主強弱、用神喜神忌神、清濁與調候。其他各區說的「喜／忌」都以課堂三步法為準。" },
  { key: "pattern", short: "格局", title: "格局・十神組合", desc: "格局高低總評（上／中／下與加減分理由）、結構與格局成敗、特別格局、十神組合。" },
  { key: "interaction", short: "合沖刑害", title: "干支作用（合沖刑害）", desc: "天干五合、地支沖合刑害、同柱干支，以及作用之後的強弱複核。" },
  { key: "family", short: "六親婚姻", title: "六親・婚姻", desc: "父母、兄弟、配偶、子女的星與宮，擇偶、婚期、轉盤看親人。" },
  { key: "health", short: "健康", title: "健康", desc: "五行太過不及、相剋的病症、需要留意的身體部位。" },
  { key: "career", short: "事業財富", title: "事業・財富・學業", desc: "適合的行業、財的多寡與來源、讀書與考運。" },
  { key: "learn", short: "學習", title: "學習", desc: "課程進度、練習模式、反驗筆記、書上相似命例、名詞表。" },
  { key: "reference", short: "量化・上課", title: "量化計分與上課參考", desc: "量化五行力的計分明細、課程流程與課堂命例比對。" },
];

/** 新手模式不顯示「量化計分與上課參考」分區。 */
export function visibleTopicList(isResearch: boolean, exclude: TopicKey[] = []) {
  return TOPICS.filter((topic) => !exclude.includes(topic.key) && (isResearch || topic.key !== "reference"));
}
