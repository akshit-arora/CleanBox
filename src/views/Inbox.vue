<template>
    <div class="h-full flex flex-col">
        <div class="p-6 pb-2">
            <h1 class="text-3xl font-bold tracking-tight">Inbox</h1>
            <p class="text-muted-foreground">All your emails in chronological order.</p>
        </div>
        
        <ScrollArea class="flex-1 p-6 pt-2">
            <div class="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead class="w-[300px]">Sender</TableHead>
                            <TableHead>Subject</TableHead>
                            <TableHead class="w-[100px]">Type</TableHead>
                            <TableHead class="w-[150px] text-right">Date</TableHead>
                            <TableHead class="w-[50px]"></TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <TableRow v-for="email in emails" :key="email.id">
                            <!-- Sender -->
                            <TableCell class="font-medium">
                                <div class="flex items-center gap-3">
                                    <Avatar>
                                        <AvatarImage src="" />
                                        <AvatarFallback>{{ getInitials(email.sender) }}</AvatarFallback>
                                    </Avatar>
                                    <span class="truncate">{{ email.sender }}</span>
                                </div>
                            </TableCell>

                            <!-- Subject -->
                            <TableCell>
                                <div class="flex flex-col max-w-[500px]">
                                    <span class="font-medium truncate">{{ email.subject }}</span>
                                    <span class="text-xs text-muted-foreground truncate">{{ email.body_preview }}</span>
                                </div>
                            </TableCell>

                            <!-- Type / Badge -->
                            <TableCell>
                                <Badge :variant="getBadgeVariant()" :class="getBadgeColor(email.view_mode)">
                                    {{ email.view_mode }}
                                </Badge>
                            </TableCell>

                            <!-- Date -->
                            <TableCell class="text-right">
                                {{ formatTime(email.received_at) }}
                            </TableCell>

                            <!-- Actions -->
                            <TableCell>
                                <Button variant="ghost" size="icon">
                                    <MoreHorizontal class="w-4 h-4" />
                                </Button>
                            </TableCell>
                        </TableRow>

                        <TableRow v-if="emails.length === 0">
                            <TableCell colspan="5" class="h-24 text-center">
                                No emails found.
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </div>
        </ScrollArea>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { MoreHorizontal } from 'lucide-vue-next'
import { formatDistanceToNow } from 'date-fns'

// UI Components
import { ScrollArea } from '@/components/ui/scroll-area'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

// Types
interface Email {
    id: string
    sender: string
    subject: string
    body_preview: string
    view_mode: string
    received_at: string
}

const emails = ref<Email[]>([])

onMounted(async () => {
    try {
        emails.value = await invoke('get_recent_emails', { limit: 50 })
    } catch (error) {
        console.error('Failed to fetch emails:', error)
    }
})

// Helpers
function getInitials(name: string) {
    return name
        .split(' ')
        .map(n => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2)
}

function formatTime(dateStr: string) {
    try {
        return formatDistanceToNow(new Date(dateStr), { addSuffix: true })
    } catch (e) {
        return dateStr
    }
}

function getBadgeVariant() {
    return 'outline' as const // using outline for all, differentiating with class
}

function getBadgeColor(mode: string) {
    switch (mode) {
        case 'LEDGER': return 'text-green-500 border-green-500 bg-green-500/10'
        case 'WORKFLOW': return 'text-blue-500 border-blue-500 bg-blue-500/10'
        case 'PULSE': return 'text-purple-500 border-purple-500 bg-purple-500/10'
        case 'FEED': return 'text-orange-500 border-orange-500 bg-orange-500/10'
        default: return 'text-gray-500 border-gray-500 bg-gray-500/10'
    }
}
</script>
