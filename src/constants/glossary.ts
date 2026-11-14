// 名詞白話解釋。畫面上的文字遇到這些詞會自動加虛線底線，點一下顯示說明與出處。
// 解釋以許羽賢老師課堂為主，其他書的說法標出書名。新增名詞時，較長的詞會優先比對（例如「歲運並臨」先於「歲運」）。

export type GlossaryEntry = { term: string; text: string; source: string; aliases?: string[] };

export const GLOSSARY: GlossaryEntry[] = [
  { term: "日主", text: "日柱的天干，代表命主自己。整張盤都是拿其他字和日主比較。", source: "各書共識", aliases: ["日元", "日干"] },
  { term: "月令", text: "月柱的地支，代表出生的季節，是判斷日主得不得時、取格局的依據。", source: "課堂 07-22" },
  { term: "十神", text: "其他字和日主的關係：生我（印）、同我（比劫）、我生（食傷）、我剋（財）、剋我（官殺），再分陰陽成十種。", source: "各書共識" },
  { term: "格局", text: "看月令藏干哪一個透出天干，取那個十神當格局名稱。格局是這張盤的「主題」，不是好壞。", source: "課堂 07-01、07-08", aliases: ["取格"] },
  { term: "得令", text: "日主在月令得到季節的幫助：月令和日主同五行，或月令生日主。", source: "課堂 07-22", aliases: ["得時"] },
  { term: "得勢", text: "八個字分兩黨：同黨（印、比劫）和異黨（食傷、財、官殺），同黨多就是得勢；平手時看月令站哪一邊。", source: "課堂 09-02" },
  { term: "得地", text: "地支藏干裡有和日主同五行的字，叫有根；有根就是得地。", source: "課堂 07-22" },
  { term: "同黨", text: "幫日主的字：印（生我）和比劫（同我）。", source: "課堂 09-02" },
  { term: "異黨", text: "耗日主力量的字：食傷（我生）、財（我剋）、官殺（剋我）。", source: "課堂 09-02" },
  { term: "強候選", text: "課堂三步法判為身強這一邊，但還要看合沖變化與大運驗證，所以叫「候選」。", source: "課堂三步法" },
  { term: "弱候選", text: "課堂三步法判為身弱這一邊，但還要看合沖變化與大運驗證，所以叫「候選」。", source: "課堂三步法" },
  { term: "身強", text: "日主力量偏多。要用食傷洩、財耗、官殺剋來平衡。", source: "課堂 07-22", aliases: ["身旺"] },
  { term: "身弱", text: "日主力量不足。要用印生、比劫幫來平衡。", source: "課堂 07-22" },
  { term: "用神", text: "這張盤最需要的那個五行，用來把日主調到平衡。先用扶抑定方向，再看調候、通關、病藥。", source: "課堂 08-05～08-26" },
  { term: "喜神", text: "生用神、或幫用神的五行，對命主也是有利的。", source: "課堂 08-26" },
  { term: "忌神", text: "剋用神、破壞平衡的五行，對命主不利。", source: "課堂 08-26" },
  { term: "仇神", text: "生忌神的五行，間接不利。", source: "課堂 08-26" },
  { term: "閒神", text: "對平衡影響不大的五行。", source: "課堂 08-26" },
  { term: "扶抑", text: "身弱就扶（印、比劫），身強就抑（食傷、財、官殺）。是取用神的基本法，其他方法不能和它衝突。", source: "課堂 08-05" },
  { term: "調候", text: "冬天太冷要火、夏天太熱要水，用來調整寒暖。課堂以扶抑為先，候選裡優先挑能調候的。", source: "課堂 08-19；陸致極、蘇民峰較重視" },
  { term: "通關", text: "兩個五行互相打架時，找一個能把它們接起來的五行（例如金木相戰用水）。", source: "課堂 08-26" },
  { term: "病藥", text: "盤裡有一個字專門傷用神（病），能制住它的字就是藥。", source: "課堂 08-12" },
  { term: "特別格局", text: "日主極弱或極旺、不能用一般扶抑看的盤，例如從格、專旺格、化氣格。條件要很嚴格才成立。", source: "課堂 07-01～07-15、09-02～09-09；蘇民峰", aliases: ["從格"] },
  { term: "邊界盤", text: "強弱接近中間的盤。各書在這種盤最容易判不一樣，要用過去大運發生的事反驗。", source: "八字諸書融會貫通指南 第一章" },
  { term: "五合", text: "天干兩兩相合：甲己、乙庚、丙辛、丁壬、戊癸。合了不一定會化，課堂分五種情形。", source: "課堂 10-07、10-14", aliases: ["天干五合"] },
  { term: "合而不化", text: "兩個天干相合但沒有變成新的五行，彼此被綁住，作用減弱。沒把握時都當合而不化。", source: "課堂 10-14" },
  { term: "合化", text: "天干相合後真的變成新的五行，條件嚴格（月令支持、沒有被沖剋）。", source: "課堂 10-14" },
  { term: "六沖", text: "地支相對的兩個字互相衝撞：子午、丑未、寅申、卯酉、辰戌、巳亥。", source: "課堂 10-21", aliases: ["相沖"] },
  { term: "六合", text: "地支兩兩相合：子丑、寅亥、卯戌、辰酉、巳申、午未。課堂說地支有合無化。", source: "課堂 10-21" },
  { term: "三合", text: "三個地支合成一局：申子辰水、亥卯未木、寅午戌火、巳酉丑金。只有兩個字叫半合。", source: "課堂 10-28", aliases: ["半合"] },
  { term: "三會", text: "同一季節的三個地支：寅卯辰木、巳午未火、申酉戌金、亥子丑水，力量比三合更大。", source: "課堂 10-28" },
  { term: "三刑", text: "寅巳申（恃勢之刑）、丑戌未（無恩之刑）、子卯（無禮之刑）。課堂不看名稱，直接讀成地支本氣的五行生剋，再看喜忌；刑在月令是加強，在日時是鬆動。", source: "課堂 11-04；講義 ch.24", aliases: ["相刑"] },
  { term: "自刑", text: "辰辰、午午、酉酉、亥亥兩個相同的地支相見。課堂讀成那個五行加強。", source: "課堂 11-04" },
  { term: "六害", text: "子未、丑午、寅巳、卯辰、申亥、酉戌。老師說害屬神煞、不足為憑，只讀成兩個地支的五行生剋。", source: "課堂 09-23、11-04", aliases: ["相害"] },
  { term: "本氣", text: "地支藏干裡力量最大的那一個。月令被刑或被沖之後只剩本氣，取格要用本氣。", source: "課堂 11-04" },
  { term: "干支同氣", text: "大運的天干、地支是同一個五行（例如庚申、甲寅）。原局有病無藥時，要走干支同氣的運才真的有用。", source: "課堂 11-04" },
  { term: "通根", text: "天干在地支裡找得到同五行的字（例如甲坐寅、戊坐辰），站得穩、有力量。同一柱干支同類最有力。", source: "講義第 25 章", aliases: ["干支同類"] },
  { term: "截腳", text: "天干坐在剋它的地支上（例如丁亥，水剋火），天干虛浮無力。", source: "講義第 25 章" },
  { term: "蓋頭", text: "天干剋它坐的地支（例如庚寅，金剋木），地支的力量被壓住一些。", source: "講義第 25 章" },
  { term: "成局", text: "三合、三會或半合成立，那個五行的力量大增。課堂只論合力、不論化，成員原本的五行還在。", source: "課堂 10-28", aliases: ["合力"] },
  { term: "藏干", text: "地支裡藏著的天干，有本氣、中氣、餘氣。本氣力量最大。", source: "各書共識" },
  { term: "透干", text: "地支藏干的字也出現在天干，叫透出，力量明顯。", source: "課堂 07-08", aliases: ["透出"] },
  { term: "大運", text: "每十年換一步的運，從月柱往前或往後排。各書多數以地支為主、天干為輔。", source: "指南 第九章" },
  { term: "流年", text: "每一年的干支。大運定十年的底色，流年定這一年的事件。", source: "指南 第九章" },
  { term: "伏吟", text: "運歲的干支和原局某柱完全相同，力量加倍，常見「重複、停滯」。", source: "陸致極 進階教程", aliases: ["反吟"] },
  { term: "歲運並臨", text: "流年和大運干支相同，該干支的力量加倍，書中常是容易出事的年。", source: "陸致極 進階教程 第四章" },
  { term: "沖提", text: "運歲沖到月支（提綱），環境變動大。", source: "陸致極 動態分析教程" },
  { term: "填實", text: "原局有半合或空缺，運歲剛好補上那個字，局就成了。", source: "陸致極 動態分析教程" },
  { term: "配偶宮", text: "日柱的地支，看婚姻關係。", source: "各書共識" },
  { term: "配偶星", text: "男命看財（妻星），女命看官殺（夫星），看配偶本人。", source: "各書共識", aliases: ["妻星", "夫星"] },
  { term: "反驗", text: "用已經發生的事（升學、結婚、生病、換工作）回頭對照大運流年，確認強弱和喜忌判斷對不對。", source: "指南 第十章" },
];

const LOOKUP = new Map<string, GlossaryEntry>();
for (const entry of GLOSSARY) {
  LOOKUP.set(entry.term, entry);
  entry.aliases?.forEach((alias) => LOOKUP.set(alias, entry));
}

export function glossaryEntry(word: string): GlossaryEntry | undefined {
  return LOOKUP.get(word);
}

// 依長度由長到短排，讓「歲運並臨」比「歲運」先比對。
const WORDS = [...LOOKUP.keys()].sort((a, b) => b.length - a.length);
const PATTERN = new RegExp(`(${WORDS.map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`, "g");

export type GlossaryPiece = { text: string; entry?: GlossaryEntry };

/** 把一段文字切成「一般文字」與「名詞」片段；同一個名詞在一段裡只標第一次，避免整段都是底線。 */
export function splitGlossary(text: string): GlossaryPiece[] {
  const pieces: GlossaryPiece[] = [];
  const marked = new Set<string>();
  let last = 0;
  for (const match of text.matchAll(PATTERN)) {
    const word = match[0];
    const entry = LOOKUP.get(word);
    const index = match.index ?? 0;
    if (!entry || marked.has(entry.term)) continue;
    marked.add(entry.term);
    if (index > last) pieces.push({ text: text.slice(last, index) });
    pieces.push({ text: word, entry });
    last = index + word.length;
  }
  if (last < text.length) pieces.push({ text: text.slice(last) });
  return pieces;
}
