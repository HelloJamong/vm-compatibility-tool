<script lang="ts">
  export type StatusBadgeTone = "danger" | "success" | "neutral" | "info";

  type Props = {
    label: string;
    tone?: StatusBadgeTone;
    className?: string;
  };

  let { label, tone = "neutral", className = "" }: Props = $props();

  const toneClasses: Record<StatusBadgeTone, string> = {
    danger: "bg-red-200 text-red-800",
    success: "bg-green-200 text-green-800",
    neutral: "bg-gray-200 text-gray-700",
    info: "bg-blue-200 text-blue-900",
  };
</script>

<span class={`text-xs font-normal px-2 py-0.5 rounded-full ${toneClasses[tone]} ${className}`}>
  {label}
</span>
