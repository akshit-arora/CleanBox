<script setup lang="ts">
import { ref, onMounted } from 'vue'
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

const loading = ref(false)
const { toast } = useToast()

onMounted(async () => {
  try {
    const savedConfig: any = await invoke('get_imap_config')
    if (savedConfig) {
      // Merge saved config into default config to keep defaults if keys are missing
      config.value = { ...config.value, ...savedConfig }
    }
  } catch (error) {
    console.error('Failed to load settings:', error)
  }
})

const saveSettings = async () => {
  loading.value = true
  try {
    await invoke('save_imap_config', { config: config.value })
    toast({
      title: 'Success',
      description: 'Settings Saved',
      variant: 'success', // Now valid
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
  <div class="flex items-center justify-center min-h-screen bg-background p-4">
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
