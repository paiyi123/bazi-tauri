import { ref, shallowRef } from "vue";
import type { QuantAnnualLuckScore, QuantLuckScore } from "../types/bazi";

type AnnualCombinationMode = "active" | "full";
type LuckColumn = { type?: string } | null;

export interface LuckDetailSection {
  title: string;
  lines: string[];
}

function formatAgeRange(startAge: number | null, endAge: number | null) {
  return startAge != null && endAge != null ? `${startAge}-${endAge}歲` : "起運歲數未定";
}

export function useLuckDetailPopover() {
  const annualCombinationMode = ref<AnnualCombinationMode>("active");
  const luckDetailPopoverVisible = ref(false);
  const luckDetailTitle = ref("");
  const luckDetailSections = ref<LuckDetailSection[]>([]);
  const luckDetailVirtualRef = shallowRef<HTMLElement>();

  function buildSections(
    overview: string[],
    interactionLines?: string[],
    scoringLines?: string[],
    details?: string,
  ) {
    const sections: LuckDetailSection[] = [{ title: "概覽", lines: overview }];
    if (interactionLines?.length) {
      sections.push({ title: "入局互動", lines: interactionLines });
    }
    if (scoringLines?.length) {
      sections.push({ title: "計分明細", lines: scoringLines });
    } else if (details) {
      sections.push({ title: "計分明細", lines: details.split("；").filter(Boolean) });
    }
    return sections;
  }

  function buildLuckDetailLines(row: QuantLuckScore) {
    const overview = row.overviewLines?.length
      ? row.overviewLines
      : [
          `${row.firstHalfLabel}：${row.firstHalfScore}`,
          `${row.secondHalfLabel}：${row.secondHalfScore}`,
          `整柱參考：${row.totalScore}`,
          `R值：${row.impactRatio}`,
          `作用後命局：${row.effectiveNatalScore}`,
          `傾向：${row.tendency}`,
        ];
    return buildSections(overview, row.interactionLines, row.scoringLines, row.details);
  }

  function buildAnnualDetailLines(row: QuantAnnualLuckScore) {
    const overview = row.overviewLines?.length
      ? row.overviewLines
      : [
          `${row.activeLuckLabel}：${row.activeLuckScore}`,
          `整個大運：${row.fullLuckScore}`,
          `流年干：${row.annualStemScore}`,
          `流年支：${row.annualBranchScore}`,
          `流年整柱：${row.annualTotalScore}`,
          `${annualCombinationMode.value === "active" ? "半運合參" : "整運合參"}：${annualCombinationMode.value === "active" ? row.combinedScore : row.fullCombinedScore}`,
          `R值：${annualCombinationMode.value === "active" ? row.impactRatio : row.fullImpactRatio}`,
          `作用後命局：${annualCombinationMode.value === "active" ? row.effectiveNatalScore : row.fullEffectiveNatalScore}`,
          `傾向：${annualCombinationMode.value === "active" ? row.tendency : row.fullTendency}`,
        ];
    return buildSections(overview, row.interactionLines, row.scoringLines, row.details);
  }

  function resolveVirtualRef(event?: MouseEvent | null) {
    const rawTarget = event?.target;
    if (rawTarget instanceof HTMLElement) {
      return rawTarget.closest("tr") || rawTarget;
    }
    const current = event?.currentTarget;
    return current instanceof HTMLElement ? current : undefined;
  }

  function openLuckDetail(row: QuantLuckScore, column?: LuckColumn, event?: MouseEvent) {
    if (column?.type === "expand") return;
    luckDetailTitle.value = `大運說明：${row.ganZhi}（${formatAgeRange(row.startAge, row.endAge)}）`;
    luckDetailSections.value = buildLuckDetailLines(row);
    luckDetailVirtualRef.value = resolveVirtualRef(event);
    if (!luckDetailVirtualRef.value) return;
    luckDetailPopoverVisible.value = true;
  }

  function openAnnualDetail(row: QuantAnnualLuckScore, column?: LuckColumn, event?: MouseEvent) {
    if (column?.type === "expand") return;
    luckDetailTitle.value = `流年／歲運說明：${row.year} ${row.ganZhi}`;
    luckDetailSections.value = buildAnnualDetailLines(row);
    luckDetailVirtualRef.value = resolveVirtualRef(event);
    if (!luckDetailVirtualRef.value) return;
    luckDetailPopoverVisible.value = true;
  }

  return {
    annualCombinationMode,
    luckDetailPopoverVisible,
    luckDetailTitle,
    luckDetailSections,
    luckDetailVirtualRef,
    openLuckDetail,
    openAnnualDetail,
  };
}
