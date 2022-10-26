<template>
  <div>
    <div class="flex w-fit mb-2" v-for="item in filterItem">
      <input type="text" class="bg-bglighter p-2 border-none outline-0 rounded" v-model="item.value"
             placeholder="/regex filter/">
      <div class="flex items-center cursor-pointer bg-bglighter rounded ml-1" @click="removeFilterItem(item)">
        <span class="material-symbols-outlined text-3xl font-bold">close</span>
      </div>
    </div>

    <div class="bg-bglighter w-fit flex items-center p-2 rounded text-xl cursor-pointer select-none"
         :class="{'text-gray': !newFilterPossible()}"
         @click="addFilterItem()">
      <span class="material-symbols-outlined">add</span>
      Add
    </div>
  </div>
</template>

<script setup lang="ts">
import FilterItem from "./FilterItem.vue";
import {ref} from "vue";
import {Filter} from "../util/filter";

const filterItem = ref<Filter[]>([]);

function addFilterItem() {
  const filterItemList = filterItem.value;

  if(!newFilterPossible())
    return;

  filterItemList.push({value: ""});
}

function newFilterPossible(): boolean {
  const filterItemList = filterItem.value;

  if(filterItemList.length >= 9)
    return false;

  if (filterItemList.length != 0 && filterItemList[filterItemList.length - 1].value === "")
    return false;

  return true;
}

function removeFilterItem(item: Filter) {
  filterItem.value = filterItem.value.filter(i => i != item);
}

</script>
