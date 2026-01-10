<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const response = ref('');
const loading = ref(false);

async function fetchEmails() {
  loading.value = true;
  response.value = '';
  try {
    const res = await invoke('fetch_inbox_top');
    response.value = JSON.stringify(res, null, 2);
  } catch (e) {
    console.error(e);
    response.value = 'Error fetching emails: ' + e;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="p-8">
    <h1 class="text-3xl font-bold mb-6 text-white">Welcome to CleanBox</h1>
    <div class="bg-gray-800 p-6 rounded-lg border border-gray-700 shadow-lg mb-6">
      <p class="text-gray-300 mb-4">Click below to fetch your latest emails.</p>
      <button 
        @click="fetchEmails"
        :disabled="loading"
        class="bg-blue-600 hover:bg-blue-500 disabled:bg-blue-800 text-white font-bold py-2 px-6 rounded focus:outline-none focus:shadow-outline transition-colors"
      >
        {{ loading ? 'Fetching...' : 'Fetch Emails' }}
      </button>
    </div>

    <div v-if="response" class="bg-gray-900 p-4 rounded border border-gray-800 overflow-auto">
      <pre class="text-green-400 text-sm font-mono">{{ response }}</pre>
    </div>
  </div>
</template>
