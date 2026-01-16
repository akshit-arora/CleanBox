<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Shield, Lock, Copy } from 'lucide-vue-next'
import { useToast } from '@/components/ui/toast/use-toast'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

interface PulseItem {
  id: string
  sender: string
  service_name: string
  otp_code: string | null
  received_at: string
}

const items = ref<PulseItem[]>([])
const { toast } = useToast()

const fetchItems = async () => {
    try {
        items.value = await invoke('get_pulse_items')
    } catch (e) {
        console.error("Failed to fetch pulse items:", e)
        // Optionally toast error
    }
}

const copyCode = async (code: string) => {
    try {
        await navigator.clipboard.writeText(code)
        toast({
            title: "Code Copied!",
            description: "Verification code copied to clipboard.",
            duration: 2000,
        })
    } catch (err) {
        console.error('Failed to copy: ', err)
        toast({
            title: "Copy Failed",
            description: "Could not copy code to clipboard.",
            variant: "destructive",
            duration: 2000,
        })
    }
}

onMounted(() => {
    fetchItems()
})
</script>

<template>
    <div class="max-w-2xl mx-auto py-10 px-4">
        <div class="flex items-center gap-3 mb-8">
            <Shield class="w-8 h-8 text-primary" />
            <h1 class="text-3xl font-bold tracking-tight">Security & OTPs</h1>
        </div>

        <div class="space-y-4">
            <Card v-for="item in items" :key="item.id" class="overflow-hidden border-l-4 border-l-primary">
                <CardContent class="p-6 flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                            <Lock class="w-6 h-6 text-primary" />
                        </div>
                        <div>
                            <h3 class="font-bold text-lg">{{ item.service_name }}</h3>
                            <p class="text-muted-foreground text-sm">Verification Code</p>
                        </div>
                    </div>

                    <div class="flex items-center gap-6">
                        <div v-if="item.otp_code" class="text-3xl font-mono tracking-[0.2em] font-bold text-foreground">
                            {{ item.otp_code }}
                        </div>
                        <div v-else class="text-muted-foreground italic text-sm">
                            No code detected
                        </div>

                        <Button 
                            v-if="item.otp_code" 
                            variant="secondary" 
                            size="icon"
                            class="h-10 w-10 hover:bg-primary hover:text-primary-foreground transition-colors"
                            @click="copyCode(item.otp_code)"
                        >
                            <Copy class="w-5 h-5" />
                        </Button>
                    </div>
                </CardContent>
            </Card>
            
            <div v-if="items.length === 0" class="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <Shield class="w-16 h-16 mb-4 opacity-20" />
                <p>No active verification codes found.</p>
            </div>
        </div>
    </div>
</template>
