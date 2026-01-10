<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const host = ref('https://www.google.com/search?q=imap.gmail.com');
const port = ref(993);
const email = ref('');
const password = ref('');
const statusMsg = ref('');
const isError = ref(false);

async function saveConfig() {
  try {
    statusMsg.value = 'Saving...';
    isError.value = false;
    await invoke('save_imap_config', {
      host: host.value,
      port: port.value,
      email: email.value,
      password: password.value,
    });
    statusMsg.value = 'Configuration saved successfully!';
  } catch (e) {
    console.error(e);
    isError.value = true;
    statusMsg.value = 'Failed to save configuration: ' + e;
  }
}
</script>

<template>
  <div class="p-8">
    <h2 class="text-2xl font-bold mb-6 text-white">Settings</h2>
    <div class="max-w-md bg-gray-800 p-6 rounded-lg border border-gray-700 shadow-lg">
      <div class="mb-4">
        <label class="block text-gray-400 text-sm font-bold mb-2">IMAP Host</label>
        <input 
          v-model="host" 
          type="text" 
          class="w-full bg-gray-900 text-white border border-gray-700 rounded py-2 px-3 leading-tight focus:outline-none focus:border-blue-500 transition-colors"
        >
      </div>
      
      <div class="mb-4">
        <label class="block text-gray-400 text-sm font-bold mb-2">Port</label>
        <input 
          v-model.number="port" 
          type="number" 
          class="w-full bg-gray-900 text-white border border-gray-700 rounded py-2 px-3 leading-tight focus:outline-none focus:border-blue-500 transition-colors"
        >
      </div>

      <div class="mb-4">
        <label class="block text-gray-400 text-sm font-bold mb-2">Email</label>
        <input 
          v-model="email" 
          type="email" 
          class="w-full bg-gray-900 text-white border border-gray-700 rounded py-2 px-3 leading-tight focus:outline-none focus:border-blue-500 transition-colors"
        >
      </div>

      <div class="mb-6">
        <label class="block text-gray-400 text-sm font-bold mb-2">Password</label>
        <input 
          v-model="password" 
          type="password" 
          class="w-full bg-gray-900 text-white border border-gray-700 rounded py-2 px-3 leading-tight focus:outline-none focus:border-blue-500 transition-colors"
        >
      </div>

      <button 
        @click="saveConfig"
        class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline transition-colors"
      >
        Save Configuration
      </button>

      <div v-if="statusMsg" class="mt-4 text-sm" :class="isError ? 'text-red-500' : 'text-green-500'">
        {{ statusMsg }}
      </div>
    </div>
  </div>
</template>
