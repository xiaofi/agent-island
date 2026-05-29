import { onBeforeUnmount, onMounted } from "vue";
import { useTaskStore } from "@/stores/taskStore";

export function useDurationTicker() {
  const taskStore = useTaskStore();
  let timer: number | undefined;

  onMounted(() => {
    timer = window.setInterval(() => taskStore.bumpClock(), 1000);
  });

  onBeforeUnmount(() => {
    if (timer) {
      window.clearInterval(timer);
    }
  });
}
