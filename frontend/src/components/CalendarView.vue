<template>
  <div class="w-full h-full" id="ec">

  </div>
</template>

<script setup lang="ts">
import Calendar from '@event-calendar/core';
import TimeGrid from '@event-calendar/time-grid';
import "@event-calendar/build/event-calendar-modern.min.css";
import {onMounted, watchEffect} from "vue";
import {CalendarEntry} from "../util/calendarentry";

const props = defineProps<{
  entries: CalendarEntry[]
}>();

let ec: Calendar;

onMounted(() => {

  ec = new Calendar({
    target: document.getElementById('ec'),
    props: {
      plugins: [TimeGrid],
      options: {
        view: 'timeGridWeek',
        allDaySlot: false,
        events: props.entries
      }
    }
  });
});

watchEffect(() => {
  ec?.getEvents().forEach((e: { id: any; }) => ec?.removeEventById(e.id));

  props.entries.forEach(e => ec?.addEvent(e));
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
