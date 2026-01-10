<script setup lang="ts">
import { ref, computed } from "vue";
import Dashboard from "./components/Dashboard.vue";
import Settings from "./components/Settings.vue";

const currentView = ref('dashboard');

const currentComponent = computed(() => {
  if (currentView.value === 'settings') return Settings;
  return Dashboard;
});

function setView(view: string) {
  currentView.value = view;
}
</script>

<template>
  <div class="flex h-screen bg-gray-900 text-white font-sans">
    <!-- Sidebar -->
    <aside class="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
      <div class="p-6">
        <h1 class="text-xl font-bold tracking-tight text-white flex items-center gap-2">
          <span class="w-3 h-3 rounded-full bg-blue-500"></span>
          CleanBox
        </h1>
      </div>
      
      <nav class="flex-1 px-4 py-2 space-y-1">
        <button 
          @click="setView('dashboard')"
          :class="[
            'w-full text-left px-4 py-2 rounded-md text-sm font-medium transition-colors',
            currentView === 'dashboard' 
              ? 'bg-gray-800 text-white' 
              : 'text-gray-400 hover:text-white hover:bg-gray-800'
          ]"
        >
          Inbox
        </button>
        <button 
          @click="setView('settings')"
          :class="[
            'w-full text-left px-4 py-2 rounded-md text-sm font-medium transition-colors',
            currentView === 'settings' 
              ? 'bg-gray-800 text-white' 
              : 'text-gray-400 hover:text-white hover:bg-gray-800'
          ]"
        >
          Settings
        </button>
      </nav>

      <div class="p-4 border-t border-gray-800">
        <p class="text-xs text-gray-500 text-center">CleanBox v0.1.0</p>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto bg-gray-900">
      <component :is="currentComponent" />
    </main>
  </div>
</template>

<style>
/* Global scrollbar styling for a cleaner look */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
::-webkit-scrollbar-track {
  background: #111; 
}
::-webkit-scrollbar-thumb {
  background: #333; 
  border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
  background: #555; 
}
</style>