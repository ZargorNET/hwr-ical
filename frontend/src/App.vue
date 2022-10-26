<template>
  <div class="w-full h-full bg-bg text-white flex flex-col items-center">
    <div class="w-3/4 h-full">
      <div class="text-center">
        <h1 class="text-6xl">HWR iCal</h1>
        <p>Hier kannst Du Filter erstellen, um Deinen Stundenplan zu filtern</p>
      </div>

      <FilterList/>

      <div class="text-center">
        <h1 class="text-6xl">Preview</h1>
      </div>

      <CalendarView />
    </div>
  </div>
</template>


<script setup lang="ts">
import FilterList from "./components/FilterList.vue";
import CalendarView from "./components/CalendarView.vue";
</script>
