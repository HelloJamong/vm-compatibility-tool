<script lang="ts">
  export type SummaryCardTone = "danger" | "success" | "neutral";

  type Props = {
    title: string;
    value: string | number;
    description: string;
    tone?: SummaryCardTone;
  };

  let { title, value, description, tone = "neutral" }: Props = $props();

  const toneClasses: Record<
    SummaryCardTone,
    { wrapper: string; title: string; value: string; description: string }
  > = {
    danger: {
      wrapper: "border-red-200 bg-red-50",
      title: "text-red-700",
      value: "text-red-800",
      description: "text-red-600",
    },
    success: {
      wrapper: "border-green-200 bg-green-50",
      title: "text-green-700",
      value: "text-green-800",
      description: "text-green-600",
    },
    neutral: {
      wrapper: "border-slate-200 bg-slate-50",
      title: "text-slate-700",
      value: "text-slate-800",
      description: "text-slate-600",
    },
  };
  function toneClass(selectedTone: SummaryCardTone) {
    return toneClasses[selectedTone];
  }
</script>

<div class={`rounded-lg border px-4 py-3 ${toneClass(tone).wrapper}`}>
  <div class={`text-xs font-semibold ${toneClass(tone).title}`}>{title}</div>
  <div class={`mt-1 text-2xl font-bold ${toneClass(tone).value}`}>{value}</div>
  <div class={`text-xs ${toneClass(tone).description}`}>{description}</div>
</div>
