// 書上相似命例：在 docs/books 筆記裡用 App 排法畫的命盤中，找日干、月令、日柱相同的例子。
// 索引 src/data/bookCases.json 由 tools/bazi-report/build-case-index.mjs 產生；計分方式同 notes.mjs similar。

export type BookCase = { book: string; file: string; line: number; heading: string; natal: string[] };
export type SimilarCase = BookCase & { score: number; why: string };

export function similarCases(natal: string[], cases: BookCase[], limit = 8): SimilarCase[] {
  if (natal.length !== 4 || natal.some((pillar) => pillar.length !== 2)) return [];
  const chars = (list: string[]) => list.flatMap((pillar) => [...pillar]);
  const seen = new Set<string>();
  return cases
    .map((item) => {
      let score = 0;
      const why: string[] = [];
      if (item.natal.join(" ") === natal.join(" ")) (score += 20), why.push("四柱相同");
      if (item.natal[2][0] === natal[2][0]) (score += 4), why.push(`日干${natal[2][0]}`);
      if (item.natal[1][1] === natal[1][1]) (score += 4), why.push(`月令${natal[1][1]}`);
      if (item.natal[2] === natal[2]) (score += 2), why.push(`日柱${natal[2]}`);
      if (item.natal[1] === natal[1]) (score += 1), why.push(`月柱${natal[1]}`);
      const pool = chars(natal);
      const shared = chars(item.natal).filter((ch) => {
        const index = pool.indexOf(ch);
        if (index < 0) return false;
        pool.splice(index, 1);
        return true;
      }).length;
      score += shared * 0.5;
      return { ...item, score, why: `${why.join("、")}${why.length ? "、" : ""}同字 ${shared}/8` };
    })
    .filter((item) => item.score >= 8)
    .sort((a, b) => b.score - a.score)
    .filter((item) => {
      const key = `${item.natal.join(" ")}|${item.book}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .slice(0, limit);
}
