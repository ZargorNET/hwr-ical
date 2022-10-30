<template>
  <div class="flex flex-col sm:flex-row justify-center w-full child:w-full">
    <select v-model="selectedCourse" class="bg-bglighter select-none outline-0 rounded cursor-pointer p-2"
            @change="semesterSelect = undefined; updateSelectedSemester(null, ''); updateSelectedCourse($event)">
      <option disabled value="undefined" selected>Studiengang</option>
      <option v-for="course in Object.keys(courses)" :key="course">{{ course }}</option>
    </select>
    <select v-model="semesterSelect"
            class="bg-bglighter select-none outline-0 rounded cursor-pointer p-2 mt-2 sm:mt-0 sm:ml-2"
            @change="updateSelectedSemester($event, selectedCourse)">
      <option disabled value="undefined" selected>Kurs</option>
      <option v-for="semester in courses[selectedCourse]" :key="semester" v-if="selectedCourse !== undefined">
        {{ semester.display_name }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import {get_courses} from "../util/fetcher";
import {ref} from "vue";
import {Semester} from "../util/semester";

const emits = defineEmits(['update:selectedCourse', 'update:selectedSemester']);

const courses = ref<Map<String, Semester[]> | any>(await get_courses());
const selectedCourse = ref();
const semesterSelect = ref();

try {
  trySetFromLocalStorage();
} catch (_) {
}


function updateSelectedSemester($event: any, selectedCourse: string | null) {
  const response = selectedCourse != null ? (courses.value as any)[selectedCourse]?.find((s: Semester) => s.display_name === $event.target.value) : undefined;
  emits('update:selectedSemester', response);
  localStorage.setItem("selectedSemester", JSON.stringify(response));
}

function updateSelectedCourse($event: any) {
  emits('update:selectedCourse', $event.target.value);
  localStorage.setItem("selectedCourse", $event.target.value)
}

function trySetFromLocalStorage() {
  const selectedCourseLS = localStorage.getItem("selectedCourse");
  const selectedSemesterLS = localStorage.getItem("selectedSemester");

  if (selectedCourseLS == null || selectedSemesterLS == null)
    return;

  const selectedSemesterObj = JSON.parse(selectedSemesterLS);

  if (courses.value[selectedCourseLS]?.some((s: Semester) => s.display_name === selectedSemesterObj.display_name)) {
    selectedCourse.value = selectedCourseLS;
    semesterSelect.value = selectedSemesterObj.display_name;

    emits('update:selectedCourse', selectedCourseLS);
    emits('update:selectedSemester', selectedSemesterObj);
  }
}
</script>
