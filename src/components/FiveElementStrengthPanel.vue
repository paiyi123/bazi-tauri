<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>用神分析</h2>
      </div>
    </template>

    <template v-if="yongShen">
      <el-descriptions :column="1" border class="analysis-box">
        <el-descriptions-item label="方法">
          {{ yongShen.methodSummary }}
        </el-descriptions-item>
        <el-descriptions-item label="判定基礎">
          {{ yongShen.strengthBasis }}；{{ yongShen.printPresence }}
        </el-descriptions-item>
        <el-descriptions-item label="可用十神">
          <div class="tag-list">
            <el-tag v-for="item in yongShen.favorableTenGods" :key="`fav-god-${item}`" type="success" effect="plain">
              {{ item }}
            </el-tag>
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="可用五行">
          <div class="tag-list">
            <el-tag v-for="item in yongShen.favorableElements" :key="`fav-element-${item}`" type="success">
              {{ item }}
            </el-tag>
          </div>
        </el-descriptions-item>
        <el-descriptions-item v-if="yongShen.conditionalElements.length" label="輔助可用">
          <div class="tag-list">
            <el-tag
              v-for="item in yongShen.conditionalElements"
              :key="`conditional-element-${item}`"
              type="warning"
              effect="plain"
            >
              {{ item }}
            </el-tag>
          </div>
          <div class="subline">對應十神：{{ yongShen.conditionalTenGods.join("、") }}</div>
        </el-descriptions-item>
        <el-descriptions-item label="忌神五行">
          <div class="tag-list">
            <el-tag v-for="item in yongShen.unfavorableElements" :key="`bad-element-${item}`" type="danger">
              {{ item }}
            </el-tag>
          </div>
          <div class="subline">對應十神：{{ yongShen.unfavorableTenGods.join("、") }}</div>
        </el-descriptions-item>
        <el-descriptions-item label="慎用">
          <div v-if="yongShen.cautionElements.length" class="tag-list">
            <el-tag v-for="item in yongShen.cautionElements" :key="`caution-element-${item}`" type="warning">
              {{ item }}
            </el-tag>
          </div>
          <span v-else>無</span>
          <div v-if="yongShen.cautionTenGods.length" class="subline">
            對應十神：{{ yongShen.cautionTenGods.join("、") }}
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="判斷過程">
          {{ yongShen.process }}
        </el-descriptions-item>
        <el-descriptions-item label="結論">
          {{ yongShen.conclusion }}
        </el-descriptions-item>
      </el-descriptions>

      <div v-if="congPattern" class="subsection-title">從與不從分析</div>
      <el-descriptions v-if="congPattern" :column="1" border class="analysis-box">
        <el-descriptions-item label="方法">
          {{ congPattern.methodSummary }}
        </el-descriptions-item>
        <el-descriptions-item label="界線">
          臨界：正負 {{ congPattern.boundaryScore }}；真從：正負 {{ congPattern.trueBoundaryScore }}
        </el-descriptions-item>
        <el-descriptions-item label="判定">
          {{ congPattern.pattern }}；{{ congPattern.authenticity }}；{{ congPattern.subtype }}
        </el-descriptions-item>
        <el-descriptions-item label="主導十神">
          {{ congPattern.dominantFamily }}
        </el-descriptions-item>
        <el-descriptions-item v-if="congPattern.primaryUseGods.length" label="主要可用">
          <div class="tag-list">
            <el-tag v-for="item in congPattern.primaryUseGods" :key="`cong-primary-${item}`" type="success">
              {{ item }}
            </el-tag>
          </div>
        </el-descriptions-item>
        <el-descriptions-item v-if="congPattern.secondaryUseGods.length" label="次要可用">
          <div class="tag-list">
            <el-tag
              v-for="item in congPattern.secondaryUseGods"
              :key="`cong-secondary-${item}`"
              type="warning"
              effect="plain"
            >
              {{ item }}
            </el-tag>
          </div>
        </el-descriptions-item>
        <el-descriptions-item v-if="congPattern.avoidGods.length" label="忌神">
          <div class="tag-list">
            <el-tag v-for="item in congPattern.avoidGods" :key="`cong-avoid-${item}`" type="danger">
              {{ item }}
            </el-tag>
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="風險提示">
          {{ congPattern.riskNote }}
        </el-descriptions-item>
        <el-descriptions-item label="判斷過程">
          {{ congPattern.process }}
        </el-descriptions-item>
        <el-descriptions-item label="結論">
          {{ congPattern.conclusion }}
        </el-descriptions-item>
      </el-descriptions>

      <div v-if="shaYin" class="subsection-title">殺印相生分析</div>
      <el-descriptions v-if="shaYin" :column="1" border class="analysis-box">
        <el-descriptions-item label="方法">
          {{ shaYin.methodSummary }}
        </el-descriptions-item>
        <el-descriptions-item label="天干鏈">
          {{ shaYin.stemChain || "未形成" }}
          <div v-if="shaYin.stemPatternFound" class="subline">
            殺分：{{ shaYin.stemSourceNegativeScore }}；印支援：{{ shaYin.stemSealSupportScore }}；
            {{ shaYin.stemTransformed ? `轉正後參考總分：${shaYin.stemAdjustedTotalScore}` : "未達轉正條件" }}
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="地支鏈">
          {{ shaYin.branchChain || "未形成" }}
          <div v-if="shaYin.branchPatternFound" class="subline">
            殺分：{{ shaYin.branchSourceNegativeScore }}；印支援：{{ shaYin.branchSealSupportScore }}；
            {{ shaYin.branchTransformed ? `轉正後參考總分：${shaYin.branchAdjustedTotalScore}` : "未達轉正條件" }}
          </div>
        </el-descriptions-item>
        <el-descriptions-item label="判斷過程">
          {{ shaYin.process }}
        </el-descriptions-item>
        <el-descriptions-item label="結論">
          {{ shaYin.conclusion }}
        </el-descriptions-item>
      </el-descriptions>
    </template>

    <template v-else>
      <el-empty description="尚未產生正式用神分析。" />
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
}>();

const yongShen = computed(() => props.result.quantModel?.yongShen);
const congPattern = computed(() => props.result.quantModel?.congPattern);
const shaYin = computed(() => props.result.quantModel?.shaYin);
</script>

<style scoped>
.analysis-box {
  margin-top: 12px;
}

.subsection-title {
  margin-top: 18px;
  font-weight: 700;
  font-size: calc(16px * var(--app-font-scale));
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.subline {
  margin-top: 8px;
  color: var(--el-text-color-secondary);
  font-size: calc(13px * var(--app-font-scale));
}

</style>
