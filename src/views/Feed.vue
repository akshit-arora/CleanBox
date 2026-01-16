<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { formatDistanceToNow } from 'date-fns'

import { Card, CardHeader, CardContent, CardFooter } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from '@/components/ui/sheet'
import { ScrollArea } from '@/components/ui/scroll-area'

interface Email {
  id: string
  sender: string
  subject: string
  body_preview: string
  body?: string
  received_at: string
  view_mode: string
}

const feedEmails = ref<Email[]>([])
const selectedEmail = ref<Email | null>(null)
const isSheetOpen = ref(false)

const getInitials = (name: string) => {
  return name
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

const formatDate = (dateStr: string) => {
  try {
    return formatDistanceToNow(new Date(dateStr), { addSuffix: true })
  } catch (e) {
    return dateStr
  }
}

const openSheet = (email: Email) => {
  selectedEmail.value = email
  isSheetOpen.value = true
}

onMounted(async () => {
  try {
    feedEmails.value = await invoke('get_emails', { viewMode: 'FEED' })
  } catch (error) {
    console.error('Failed to fetch feed emails:', error)
  }
})
</script>

<template>
  <div class="container max-w-3xl mx-auto py-8 px-4">
    <div class="space-y-6">
      <div v-if="feedEmails.length === 0" class="text-center py-10 text-muted-foreground">
        No newsletters found.
      </div>

      <Card
        v-for="email in feedEmails"
        :key="email.id"
        class="cursor-pointer hover:shadow-md transition-shadow"
        @click="openSheet(email)"
      >
        <CardHeader>
          <div class="flex items-center justify-between mb-2">
            <Badge variant="secondary">Newsletter</Badge>
            <span class="text-xs text-muted-foreground whitespace-nowrap ml-2">
              {{ formatDate(email.received_at) }}
            </span>
          </div>
          
          <div class="flex items-center gap-3 mb-1">
            <Avatar class="h-8 w-8">
              <AvatarImage src="" />
              <AvatarFallback>{{ getInitials(email.sender) }}</AvatarFallback>
            </Avatar>
            <span class="text-sm font-medium">{{ email.sender }}</span>
          </div>

          <h3 class="text-xl font-bold leading-tight mt-2">
            {{ email.subject }}
          </h3>
        </CardHeader>

        <CardContent>
          <p class="text-muted-foreground line-clamp-3">
            {{ email.body_preview }}
          </p>
        </CardContent>

        <CardFooter>
          <Button variant="outline" class="w-full sm:w-auto" @click.stop="openSheet(email)">
            Read Full Issue
          </Button>
        </CardFooter>
      </Card>
    </div>

    <!-- Reader Sheet -->
    <Sheet v-model:open="isSheetOpen">
      <SheetContent class="w-full sm:max-w-xl md:max-w-2xl overflow-hidden flex flex-col p-0">
        <div class="p-6 border-b shrink-0 bg-background z-10">
          <SheetHeader>
            <SheetTitle class="text-xl leading-snug">
              {{ selectedEmail?.subject }}
            </SheetTitle>
            <SheetDescription class="flex items-center gap-2 mt-2">
              <span class="font-medium text-foreground">{{ selectedEmail?.sender }}</span>
              <span>•</span>
              <span>{{ selectedEmail ? formatDate(selectedEmail.received_at) : '' }}</span>
            </SheetDescription>
          </SheetHeader>
        </div>
        
        <ScrollArea class="h-full">
            <div class="p-6">
                <!-- HTML Content Renderer -->
                <div 
                    v-if="selectedEmail?.body"
                    class="prose dark:prose-invert max-w-none prose-sm sm:prose-base prose-img:rounded-md prose-a:text-blue-500 break-words"
                    v-html="selectedEmail.body"
                ></div>
                <div v-else class="text-center py-10 text-muted-foreground">
                    <p>No content available.</p>
                    <p class="text-xs mt-2">(Try re-fetching emails)</p>
                </div>
            </div>
        </ScrollArea>
      </SheetContent>
    </Sheet>
  </div>
</template>


