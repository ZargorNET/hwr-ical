<template>
  <div>
    <div class="flex w-fit mb-2" v-for="item in filterItems">
      <input type="text" class="bg-bglighter p-2 border-none outline-0 rounded" v-model="item.value"
             placeholder="/regex filter/" @input="updateEmit(); addFilterItem();" @focusin="item.focused = true"
             @focusout="item.focused = false; addFilterItem();">
      <div class="flex items-center cursor-pointer bg-bglighter rounded ml-1" @click="removeFilterItem(item)">
        <span class="material-symbols-outlined text-3xl font-bold">close</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref} from "vue";
import {Filter} from "../util/filter";
import {get_regex_limit} from "../util/fetcher";

const filterItems = ref<Filter[]>([{value: "", focused: false}]);

const emits = defineEmits(["update:filterItems"]);

const maxRegex = await get_regex_limit();

function addFilterItem() {
  filterItems.value = filterItems.value.filter(i => i.value !== "" || i.focused);

  if (newFilterPossible())
    filterItems.value.push({value: "", focused: false});
}

function newFilterPossible(): boolean {
  const filterItemList = filterItems.value;

  if (filterItemList.length >= maxRegex)
    return false;

  if (filterItemList.length != 0 && filterItemList[filterItemList.length - 1].value === "")
    return false;

  return true;
}

function removeFilterItem(item: Filter) {
  filterItems.value = filterItems.value.filter(i => i != item);
  addFilterItem();
  updateEmit();
}

function updateEmit() {
  emits('update:filterItems', filterItems.value);
}

</script>
