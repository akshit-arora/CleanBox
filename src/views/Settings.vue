<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '@/components/ui/toast/use-toast'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const config = ref({
  imap_host: 'imap.gmail.com',
  imap_port: '993',
  imap_user: '',
  imap_password: '', // Will be empty or masked
  smtp_host: 'smtp.gmail.com',
  smtp_port: '587',
  smtp_user: '',
  smtp_password: ''
})

// Extended interface for frontend stability
interface KanbanColumn {
    id: string;
    title: string;
    color: string;
    _key: string; // Stable internal key for Vue
}

const kanbanColumns = ref<Array<KanbanColumn>>([])

const loading = ref(false)
const { toast } = useToast()

const colorOptions = [
    { label: 'Blue', value: 'bg-blue-500/10 text-blue-500', bgClass: 'bg-blue-500' },
    { label: 'Green', value: 'bg-green-500/10 text-green-500', bgClass: 'bg-green-500' },
    { label: 'Orange', value: 'bg-orange-500/10 text-orange-500', bgClass: 'bg-orange-500' },
    { label: 'Red', value: 'bg-red-500/10 text-red-500', bgClass: 'bg-red-500' },
    { label: 'Yellow', value: 'bg-yellow-500/10 text-yellow-500', bgClass: 'bg-yellow-500' },
    { label: 'Purple', value: 'bg-purple-500/10 text-purple-500', bgClass: 'bg-purple-500' },
    { label: 'Gray', value: 'bg-gray-500/10 text-gray-500', bgClass: 'bg-gray-500' },
    { label: 'Pink', value: 'bg-pink-500/10 text-pink-500', bgClass: 'bg-pink-500' },
]

onMounted(async () => {
  try {
    const savedConfig: any = await invoke('get_imap_config')
    if (savedConfig) {
      config.value = { ...config.value, ...savedConfig }
    }
    
    // Fetch Kanban Config
    const savedKanban: any = await invoke('get_kanban_config')
    let loadedColumns: any[] = [];
    
    if (savedKanban && savedKanban.length > 0) {
        loadedColumns = savedKanban
    } else {
        // Fallback defaults if for some reason backend returns empty
        loadedColumns = [
            { id: 'inbox', title: 'Inbox', color: 'bg-blue-500/10 text-blue-500' },
            { id: 'action', title: 'Action', color: 'bg-orange-500/10 text-orange-500' },
            { id: 'waiting', title: 'Waiting', color: 'bg-yellow-500/10 text-yellow-500' },
            { id: 'done', title: 'Done', color: 'bg-green-500/10 text-green-500' }
        ]
    }
    
    // Assign stable keys
    kanbanColumns.value = loadedColumns.map(col => ({
        ...col,
        _key: crypto.randomUUID()
    }));

  } catch (error) {
    console.error('Failed to load settings:', error)
  }
})

const generateId = (title: string): string => {
    return title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
}

const addKanbanColumn = () => {
    kanbanColumns.value.push({
        id: '', // Will be generated from title
        title: 'New Column', 
        color: 'bg-gray-500/10 text-gray-500',
        _key: crypto.randomUUID()
    })
    updateIds();
}

const removeKanbanColumn = (index: number) => {
    kanbanColumns.value.splice(index, 1)
}

const moveColumn = (index: number, direction: 'up' | 'down') => {
    if (direction === 'up' && index > 0) {
        const item = kanbanColumns.value.splice(index, 1)[0];
        kanbanColumns.value.splice(index - 1, 0, item);
    } else if (direction === 'down' && index < kanbanColumns.value.length - 1) {
        const item = kanbanColumns.value.splice(index, 1)[0];
        kanbanColumns.value.splice(index + 1, 0, item);
    }
}

// Watch titles to auto-update IDs
const updateIds = () => {
    const ids = new Set<string>();
    kanbanColumns.value.forEach(col => {
        let baseId = generateId(col.title);
        if (!baseId) baseId = 'col';
        
        let uniqueId = baseId;
        let counter = 1;

        while (ids.has(uniqueId)) {
            uniqueId = `${baseId}-${counter}`;
            counter++;
        }
        ids.add(uniqueId);
        
        if (col.id !== uniqueId) {
             col.id = uniqueId;
        }
    });
}

// Watch for title changes to update IDs
watch(() => kanbanColumns.value.map(c => c.title), () => {
    updateIds();
}, { deep: true })

const saveSettings = async () => {
  loading.value = true
  updateIds(); // Final consistency check
  
  try {
    await invoke('save_imap_config', { config: config.value })
    
    // Strip _key before sending to backend
    const columnsToSave = kanbanColumns.value.map(({ _key, ...rest }) => rest);
    await invoke('save_kanban_config', { columns: columnsToSave })
    
    toast({
      title: 'Success',
      description: 'Settings Saved',
      variant: 'default',
    })
  } catch (error) {
    console.error('Failed to save settings:', error)
    toast({
      title: 'Error',
      description: 'Error saving settings: ' + error,
      variant: 'destructive',
    })
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="flex items-center justify-center min-h-screen bg-background p-4 pb-20">
    <Card class="w-full max-w-2xl mx-auto mt-10">
      <CardHeader>
        <CardTitle>Connection Settings</CardTitle>
        <CardDescription>Manage your IMAP & SMTP credentials.</CardDescription>
      </CardHeader>
      <CardContent>
        <form @submit.prevent="saveSettings" class="space-y-6">
          <div class="space-y-4">
            <h3 class="text-lg font-medium">IMAP Configuration</h3>
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label for="imap_host">Host</Label>
                <Input id="imap_host" v-model="config.imap_host" placeholder="imap.example.com" />
              </div>
              <div class="space-y-2">
                <Label for="imap_port">Port</Label>
                <Input id="imap_port" v-model="config.imap_port" placeholder="993" />
              </div>
              <div class="space-y-2">
                <Label for="imap_user">Email / User</Label>
                <Input id="imap_user" v-model="config.imap_user" placeholder="user@example.com" />
              </div>
              <div class="space-y-2">
                <Label for="imap_password">Password</Label>
                <Input id="imap_password" type="password" v-model="config.imap_password" placeholder="••••••••" />
              </div>
            </div>
          </div>


          <div class="space-y-4">
            <h3 class="text-lg font-medium">SMTP Configuration</h3>
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label for="smtp_host">Host</Label>
                <Input id="smtp_host" v-model="config.smtp_host" placeholder="smtp.example.com" />
              </div>
              <div class="space-y-2">
                <Label for="smtp_port">Port</Label>
                <Input id="smtp_port" v-model="config.smtp_port" placeholder="587" />
              </div>
              <div class="space-y-2">
                <Label for="smtp_user">Email / User</Label>
                <Input id="smtp_user" v-model="config.smtp_user" placeholder="user@example.com" />
              </div>
              <div class="space-y-2">
                <Label for="smtp_password">Password</Label>
                <Input id="smtp_password" type="password" v-model="config.smtp_password" placeholder="••••••••" />
              </div>
            </div>
          </div>

          <div class="space-y-4 pt-4 border-t">
            <div class="flex items-center justify-between">
                <div>
                    <h3 class="text-lg font-medium">Kanban Board Columns</h3>
                    <p class="text-xs text-muted-foreground">Customize and reorder your workflow stages.</p>
                </div>
                <Button type="button" variant="outline" size="sm" @click="addKanbanColumn">
                    + Add Column
                </Button>
            </div>
            
            <div class="space-y-3">
                <TransitionGroup name="list" tag="div" class="space-y-3">
                    <div 
                        v-for="(col, index) in kanbanColumns" 
                        :key="col._key" 
                        class="flex flex-col sm:flex-row gap-4 items-start sm:items-end p-3 border rounded-lg bg-card/50 transition-all duration-300"
                    >
                         <!-- Drag Handle Removed -->

                        <div class="flex-1 space-y-1 w-full">
                            <Label>Title</Label>
                            <Input v-model="col.title" placeholder="Column Title" @input="updateIds" />
                        </div>
                        
                        <div class="space-y-1">
                             <Label class="mb-2 block">Color</Label>
                             <div class="flex items-center gap-1.5 flex-wrap">
                                <button 
                                    type="button"
                                    v-for="opt in colorOptions" 
                                    :key="opt.value"
                                    @click="col.color = opt.value"
                                    :class="[
                                        'w-6 h-6 rounded-full border-2 transition-all',
                                        opt.bgClass,
                                        col.color === opt.value ? 'border-primary scale-110 shadow-sm' : 'border-transparent opacity-70 hover:opacity-100 hover:scale-105'
                                    ]"
                                    :title="opt.label"
                                />
                             </div>
                        </div>

                        <div class="flex flex-col justify-end gap-1 pb-0.5">
                             <div class="flex gap-1">
                                <Button type="button" variant="ghost" size="icon" @click="moveColumn(index, 'up')" :disabled="index === 0" title="Move Up" class="h-6 w-6">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-up"><path d="m18 15-6-6-6 6"/></svg>
                                </Button>
                                <Button type="button" variant="ghost" size="icon" @click="moveColumn(index, 'down')" :disabled="index === kanbanColumns.length - 1" title="Move Down" class="h-6 w-6">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
                                </Button>
                             </div>
                            
                            <Button type="button" variant="ghost" size="icon" @click="removeKanbanColumn(index)" :disabled="kanbanColumns.length <= 1" class="text-muted-foreground hover:text-destructive h-8 w-8 self-end">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash-2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
                            </Button>
                        </div>
                    </div>
                </TransitionGroup>
            </div>
            <!-- Preview of IDs for power users or debugging, hidden for now -->
            <div class="text-[10px] text-muted-foreground/50 mt-2">
                IDs: {{ kanbanColumns.map(c => c.id).join(', ') }}
            </div>
          </div>
        </form>
      </CardContent>
      <CardFooter>
        <Button type="button" @click="saveSettings" :disabled="loading" class="w-full">
          {{ loading ? 'Saving...' : 'Save Configuration' }}
        </Button>
      </CardFooter>
    </Card>
  </div>
</template>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(30px);
}

.dashed-border {
    border-style: dashed;
    border-color: hsl(var(--primary));
}
</style>
