<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Send } from 'lucide-vue-next'

// Types
interface ChatThread {
    sender_name: string
    sender_email: string
    latest_subject: string
    last_message_time: string
}

interface Email {
    id: string
    sender: string
    subject: string
    body_preview: string
    received_at: string
}

// State
const threads = ref<ChatThread[]>([])
const activeThread = ref<ChatThread | null>(null)
const messages = ref<Email[]>([])
const newMessage = ref('')

// Load Threads
const loadThreads = async () => {
    try {
        threads.value = await invoke('get_chat_threads')
    } catch (error) {
        console.error('Failed to load threads:', error)
    }
}

// Select Thread
const selectThread = async (thread: ChatThread) => {
    activeThread.value = thread
    try {
        messages.value = await invoke('get_thread_messages', { email: thread.sender_email })
    } catch (error) {
        console.error('Failed to load messages:', error)
    }
}

// Format Date helper
const formatDate = (dateStr: string) => {
    const date = new Date(dateStr)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const getInitials = (name: string) => {
    return name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
}

onMounted(() => {
    loadThreads()
})
</script>

<template>
    <div class="flex h-screen w-full bg-background text-foreground">
        <!-- Sidebar -->
        <div class="flex h-full w-80 flex-col border-r border-border">
            <div class="p-4 font-semibold text-xl">Messages</div>
            <Separator />
            <ScrollArea class="flex-1">
                <div class="flex flex-col gap-2 p-2">
                    <button
                        v-for="thread in threads"
                        :key="thread.sender_email"
                        @click="selectThread(thread)"
                        :class="[
                            'flex items-center gap-3 rounded-lg p-3 text-left transition-colors hover:bg-muted/50',
                            activeThread?.sender_email === thread.sender_email ? 'bg-muted' : ''
                        ]"
                    >
                        <Avatar>
                            <AvatarImage src="" />
                            <AvatarFallback>{{ getInitials(thread.sender_name) }}</AvatarFallback>
                        </Avatar>
                        <div class="flex flex-col overflow-hidden">
                            <span class="truncate font-medium">{{ thread.sender_name }}</span>
                            <span class="truncate text-xs text-muted-foreground">{{ thread.latest_subject }}</span>
                        </div>
                    </button>
                </div>
            </ScrollArea>
        </div>

        <!-- Conversation -->
        <div class="flex flex-1 flex-col overflow-hidden">
            <div v-if="activeThread" class="flex h-14 items-center border-b border-border px-4 shrink-0">
                <div class="font-medium">{{ activeThread.sender_name }}</div>
                <div class="ml-2 text-sm text-muted-foreground">&lt;{{ activeThread.sender_email }}&gt;</div>
            </div>
            
            <ScrollArea v-if="activeThread" class="flex-1 p-4">
                <div class="flex flex-col gap-4">
                    <div v-for="msg in messages" :key="msg.id" class="flex flex-col items-start gap-1">
                        <div class="max-w-[70%] rounded-lg bg-muted p-3 text-sm">
                            <div class="font-semibold text-xs mb-1" v-if="msg.subject">{{ msg.subject }}</div>
                            {{ msg.body_preview }}
                        </div>
                        <span class="text-[10px] text-muted-foreground">{{ formatDate(msg.received_at) }}</span>
                    </div>
                </div>
            </ScrollArea>
            
            <div v-else class="flex flex-1 items-center justify-center text-muted-foreground">
                Select a conversation to start chatting
            </div>

            <!-- Footer -->
             <div v-if="activeThread" class="border-t border-border p-4 shrink-0">
                <div class="flex gap-2">
                    <Textarea v-model="newMessage" placeholder="Type a message..." class="min-h-[40px] max-h-[120px]" />
                    <Button variant="ghost" size="icon">
                        <Send class="h-4 w-4" />
                    </Button>
                </div>
            </div>
        </div>
    </div>
</template>
