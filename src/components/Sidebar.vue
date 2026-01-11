<script setup lang="ts">
import { useRoute } from 'vue-router'
import { LayoutDashboard, MessageSquare, Newspaper, Wallet, Activity, Settings, Inbox } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

const route = useRoute()

const navigation = [
  { name: 'Kanban', href: '/kanban', icon: LayoutDashboard },
  { name: 'Chat', href: '/chat', icon: MessageSquare },
  { name: 'Feed', href: '/feed', icon: Newspaper },
  { name: 'Ledger', href: '/ledger', icon: Wallet },
  { name: 'Pulse', href: '/pulse', icon: Activity },
  { name: 'Inbox', href: '/inbox', icon: Inbox },
]

const isActive = (path: string) => route.path.startsWith(path)
</script>

<template>
  <div class="flex h-full w-64 flex-col border-r bg-background">
    <div class="p-6">
      <h1 class="text-xl font-bold tracking-tight">CleanBox</h1>
    </div>
    
    <div class="flex-1 px-4 space-y-2">
      <router-link
        v-for="item in navigation"
        :key="item.name"
        :to="item.href"
      >
        <Button
          :variant="isActive(item.href) ? 'secondary' : 'ghost'"
          class="w-full justify-start mb-1"
        >
          <component :is="item.icon" class="mr-2 h-4 w-4" />
          {{ item.name }}
        </Button>
      </router-link>
    </div>

    <div class="p-4 mt-auto border-t">
      <router-link to="/settings">
        <Button
          :variant="isActive('/settings') ? 'secondary' : 'ghost'"
          class="w-full justify-start"
        >
          <Settings class="mr-2 h-4 w-4" />
          Settings
        </Button>
      </router-link>
    </div>
  </div>
</template>
