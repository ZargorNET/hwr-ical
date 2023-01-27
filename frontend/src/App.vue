<template>
  <div class="w-full h-full bg-bg text-white flex flex-col items-center">
    <div class="w-3/4 h-full">
      <div class="text-center">
        <h1 class="text-6xl">HWR iCal</h1>
        <p>Mit diesem Tool kannst Du einen Link von Deinem Stundenplan generieren, welchen du in z.B. Google Kalendar
          einfügen und gleichzeitig Kurse rausfiltern kannst, welche du nicht belegst</p>
        <p>Funktioniert am besten auf einem größeren Display!</p>
      </div>

      <div class="flex items-center flex-col">
        <div class="child:p-2 flex flex-col items-center">
          <div class="sm:w-3/4">
            <h2 class="text-3xl text-center mb-2">Wähle deinen Kurs</h2>
            <Suspense>
              <CourseSelector v-model:selectedCourse="selectedCourse" v-model:selectedSemester="selectedSemester"/>
            </Suspense>
          </div>

          <div>
            <h2 class="text-3xl text-center">Blacklist</h2>
            <p class="text-sm text-gray text-center mb-2">
              Filtere Kurse raus, die Du nicht sehen willst.
              <br/>
              Als Beispiel "Englisch" um alle Kurse rauszufiltern, wo das Wort "Englisch" vorkommt.
              <br/>
              Das ganze ist ein ein <a
                href="https://medium.com/factory-mind/regex-cookbook-most-wanted-regex-aa721558c3c1"
                target="_blank"
                class="text-blurplelight">Regex String</a> (ohne Lookahead & Lookbehind), sodass man auch bestimmte
              Muster exkludieren kann.
            </p>

            <div class="flex flex-col items-center">
              <Suspense>
                <FilterList v-model:filterItems="filterItems"/>
              </Suspense>
            </div>
          </div>

        </div>
        <button
            class="disabled:bg-gray generateGradient text-white p-2 cursor-pointer text-xl rounded hover:scale-105 w-48"
            @click="generate()" :disabled="selectedCourse == null || selectedSemester == null">
          Generieren
        </button>
        <div class="text-3xl text-center mt-2">Ergebnis</div>
        <p class="text-sm text-center text-gray">Diesen Link kannst du am Laptop in z.B. <a class="text-blurplelight"
                                                                                            href="https://media.discordapp.net/attachments/915894939441848331/1017036724544749680/unknown.png"
                                                                                            target="_blank">Google
          Kalendar</a> importieren.
          <br/>
          Der Stundenplan wird dann automatisch aktualisiert!</p>
        <div class="w-full bg-bglight p-4 mt-4 flex items-center" @click="selectResultDiv()">
          <p class="w-max overflow-auto" ref="resultElement">{{ resultUrl }}</p>
          <span class="material-symbols-outlined select-none cursor-pointer pl-1 ml-auto"
                @click="selectResultDiv(); copyResultToClipboard();">content_copy</span>
        </div>
      </div>

      <div class="text-center mb-4 mt-8">
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
const resultUrl = ref<string>("-- Ergebnis --");
const resultElement = ref<HTMLParagraphElement>();

async function generate() {
  let url = `${API_URL}/${selectedCourse.value.toLowerCase()}/${selectedSemester.value!.year_part.toLowerCase()}/${selectedSemester.value!.course_part}`;

  for (let filter of filterItems.value) {
    if (filter.value !== "")
      url += `/${encodeURI(filter.value)}`;
  }

  url += "/";

  console.info(`Generated URL ${url}`);
  resultUrl.value = url;

  await updateCalendar(url + "?new=false");
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
}

function selectResultDiv() {
  const range = document.createRange();
  range.selectNode(resultElement.value!);
  window.getSelection()!.removeAllRanges();
  window.getSelection()!.addRange(range);
}

function copyResultToClipboard() {
  document.execCommand('copy');
}

</script>


<style>
.generateGradient:not(:disabled) {
  background: linear-gradient(to right, #12c2e9, #c471ed, #f64f59);
}
</style>
