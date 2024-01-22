<template>
  <div>
    <div class="flex w-fit mb-2" v-for="item in filterItems">
      <input type="text" class="bg-bglighter p-2 border-none outline-0 rounded" v-model="item.value"
             placeholder="/regex filter/" @input="updateEmit();" @focusin="item.focused = true"
             @focusout="item.focused = false; updateEmit();">
      <div class="flex items-center cursor-pointer bg-bglighter rounded ml-1" @click="removeFilterItem(item)">
        <span class="material-symbols-outlined text-3xl font-bold">close</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {onMounted, ref} from "vue";
import {Filter} from "../util/filter";
import {get_regex_limit} from "../util/fetcher";

const filterItems = ref<Filter[]>(JSON.parse(localStorage.getItem("filter") ?? '[{"value": "", "focused": false}]'));
const emits = defineEmits(["update:filterItems"]);
const maxRegex = await get_regex_limit();

onMounted(() => updateEmit());

function updateFilterListCount() {
  filterItems.value = filterItems.value.filter(i => i.focused || i.value !== "");

  if (newFilterPossible())
    filterItems.value.push({value: "", focused: false});

  localStorage.setItem("filter", JSON.stringify(filterItems.value));
}

function newFilterPossible(): boolean {
  const filterItemList = filterItems.value;

  if (filterItemList.length > maxRegex)
    return false;

  if (filterItemList.length != 0 && filterItemList[filterItemList.length - 1].value === "")
    return false;

  return true;
}

function removeFilterItem(item: Filter) {
  filterItems.value = filterItems.value.filter(i => i != item);
  updateEmit();
}

function updateEmit() {
  updateFilterListCount();
  emits('update:filterItems', filterItems.value);
}

</script>
