<template>
  <div class="w-full h-full bg-bg text-white flex flex-col items-center">
    <div class="w-3/4 h-full">
      <div class="text-center">
        <h1 class="text-6xl">HWR iCal</h1>
        <p>Hier kannst Du Filter erstellen, um Deinen Stundenplan zu filtern</p>
      </div>

      <div>
        <h2 class="text-xl">Wähle deinen Kurs</h2>
        <Suspense>
          <CourseSelector v-model:selectedCourse="selectedCourse" v-model:selectedSemester="selectedSemester"/>
        </Suspense>
        <h2 class="text-xl">Filter</h2>
        <Suspense>
          <FilterList v-model:filterItems="filterItems"/>
        </Suspense>
        <button class="bg-fuchsia text-white p-2 cursor-pointer text-xl rounded hover:scale-105 disabled:bg-gray"
                @click="generate()" :disabled="selectedCourse == null || selectedSemester == null">
          Generieren
        </button>
      </div>

      <div class="text-center mb-4">
        <h1 class="text-6xl">Preview</h1>
      </div>

      <CalendarView :entries="calendarEntires"/>
    </div>
  </div>
</template>


<script setup lang="ts">
import FilterList from "./components/FilterList.vue";
import CalendarView from "./components/CalendarView.vue";
import CourseSelector from "./components/CourseSelector.vue";
import {ref} from "vue";
import {API_URL} from "./util/fetcher";
import {Semester} from "./util/semester";
import {Filter} from "./util/filter";
import * as ical from "ical";
import {CalendarEntry} from "./util/calendarentry";

const selectedCourse = ref();
const selectedSemester = ref<Semester | null>();
const filterItems = ref<Filter[]>([]);
const calendarEntires = ref<CalendarEntry[]>([]);

async function generate() {
  let url = `${API_URL}/${selectedCourse.value.toLowerCase()}/${selectedSemester.value!.year_part.toLowerCase()}/${selectedSemester.value!.course_part}`;

  for (let filter of filterItems.value) {
    url += `/${encodeURI(filter.value)}`;
  }

  url += "/";

  console.info(`Generated URL ${url}`);

  await updateCalendar(url);
}

async function updateCalendar(url: string) {
  const ics = await (await fetch(url)).text();
  const parsedIcs = ical.parseICS(ics);

  calendarEntires.value = Object.values(parsedIcs).map(entry => {
    return {
      id: entry.uid,
      start: entry.start,
      end: entry.end,
      title: entry.summary
    } as CalendarEntry;
  });

  console.log(parsedIcs);
}

</script>
