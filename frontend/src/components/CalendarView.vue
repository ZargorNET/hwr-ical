<template>
  <div class="w-full h-full" id="ec">

  </div>
</template>

<script setup lang="ts">
import Calendar from '@event-calendar/core';
import TimeGrid from '@event-calendar/time-grid';
import "@event-calendar/build/event-calendar-modern.min.css";
import {defineComponent, onMounted} from "vue";

const month = new Date().getMonth();
const year = new Date().getFullYear();

onMounted(() => {

  let ec = new Calendar({
    target: document.getElementById('ec'),
    props: {
      plugins: [TimeGrid],
      options: {
        view: 'timeGridWeek',
        allDaySlot: false,
        events: [
          {
            id: "1",
            start: new Date(),
            end: new Date(new Date().getTime() + 2 * 60 * 60 * 1000),
            title: "Test"
          }
        ]
      }
    }
  });
});
</script>

<style>
.ec {
  @apply bg-bglight rounded p-2;
}

.ec-day.ec-today {
  @apply bg-bglighter;
}

.ec-body, .ec-header {
  @apply border-none;
}

.ec-day {
  @apply border-bglightest;
}

.ec-line:not(:first-child):after {
  @apply border-b-bglightest;
}

.ec-button {
  @apply bg-bglighter text-white;
}

.ec-icon.ec-prev:after, .ec-icon.ec-next:after {
  @apply border-white;
}
.ec-button:not(:disabled) {
  @apply text-gray;
}
.ec-button:disabled {
  @apply bg-red;
}

</style>
