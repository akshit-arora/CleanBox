<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import draggable from 'vuedraggable';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';

interface Email {
    id: string;
    sender: string;
    subject: string;
    body_preview: string;
    view_mode: string;
    kanban_status: string;
    received_at: string;
}

interface KanbanColumnData {
    id: string;
    title: string;
    color: string;
    emails: Email[];
}

interface KanbanBoard {
    columns: KanbanColumnData[];
}

const board = ref<KanbanBoard>({
    columns: []
});

const getInitials = (name: string) => {
    return name
        .split(' ')
        .map((n) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2);
};

const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = (now.getTime() - date.getTime()) / 1000; // seconds

    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
};

const fetchBoard = async () => {
    try {
        const data: KanbanBoard = await invoke('get_kanban_board');
        board.value = data;
    } catch (error) {
        console.error('Failed to fetch kanban board:', error);
    }
};

const handleDragChange = async (event: any, newStatus: string) => {
    if (event.added) {
        const email = event.added.element;
        try {
            // Optimistic update handled by vuedraggable, but we need to persist it
            // Status is now the column ID (e.g., 'inbox', 'action', 'col_123')
            await invoke('update_email_status', { id: email.id, status: newStatus });
        } catch (error) {
            console.error('Failed to update email status:', error);
            // Revert changes if needed (complexity omitted for MVP)
            await fetchBoard();
        }
    }
};

onMounted(() => {
    fetchBoard();
});
</script>

<template>
    <div class="flex h-full gap-4 p-6 overflow-x-auto bg-background">
        <div v-for="column in board.columns" :key="column.id" class="flex flex-col flex-shrink-0 w-80 gap-4">
            <!-- Column Header -->
            <div class="flex items-center justify-between px-2">
                <h3 class="font-semibold text-foreground">{{ column.title }}</h3>
                <Badge :class="column.color">{{ column.emails.length }}</Badge>
            </div>

            <!-- Droppable Area -->
            <div class="flex-1 p-2 rounded-lg bg-muted/30 min-h-[500px]">
                <draggable
                    v-model="column.emails"
                    group="emails"
                    item-key="id"
                    ghost-class="opacity-50"
                    class="flex flex-col gap-3 min-h-full"
                    @change="(e: any) => handleDragChange(e, column.id)"
                >
                    <template #item="{ element }">
                        <Card class="cursor-grab hover:shadow-md transition-shadow bg-card">
                            <CardContent class="p-4 space-y-3">
                                <!-- Header: Avatar + Sender -->
                                <div class="flex items-center gap-3">
                                    <Avatar class="w-8 h-8">
                                        <AvatarImage src="" />
                                        <AvatarFallback>{{ getInitials(element.sender) }}</AvatarFallback>
                                    </Avatar>
                                    <span class="text-sm font-bold truncate text-card-foreground">{{ element.sender }}</span>
                                </div>

                                <!-- Body: Subject -->
                                <p class="text-sm font-medium leading-snug text-foreground/90 line-clamp-2">
                                    {{ element.subject }}
                                </p>
                            </CardContent>
                            <CardFooter class="px-4 py-2 border-t bg-muted/20">
                                <span class="text-xs text-muted-foreground">{{ formatTime(element.received_at) }}</span>
                            </CardFooter>
                        </Card>
                    </template>
                </draggable>
            </div>
        </div>
    </div>
</template>
