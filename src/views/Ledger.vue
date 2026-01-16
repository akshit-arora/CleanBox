<template>
    <div class="h-full flex flex-col space-y-4 p-6">
        <!-- Header -->
        <div class="space-y-1">
            <h1 class="text-3xl font-bold tracking-tight">Finance Dashboard</h1>
            <p class="text-muted-foreground">Overview of your financial emails.</p>
        </div>

        <!-- Heads-Up Display -->
        <div class="grid gap-4 md:grid-cols-3">
            <Card>
                <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle class="text-sm font-medium">
                        Total Income
                    </CardTitle>
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        class="h-4 w-4 text-muted-foreground"
                    >
                        <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                    </svg>
                </CardHeader>
                <CardContent>
                    <div class="text-2xl font-bold text-green-500">+ {{ formatMoney(stats.total_income) }}</div>
                </CardContent>
            </Card>
            <Card>
                <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle class="text-sm font-medium">
                        Total Expenses
                    </CardTitle>
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        class="h-4 w-4 text-muted-foreground"
                    >
                        <rect width="20" height="14" x="2" y="5" rx="2" />
                        <path d="M2 10h20" />
                    </svg>
                </CardHeader>
                <CardContent>
                    <div class="text-2xl font-bold text-red-500">- {{ formatMoney(stats.total_expense) }}</div>
                </CardContent>
            </Card>
            <Card>
                <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle class="text-sm font-medium">
                        Net Balance
                    </CardTitle>
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        class="h-4 w-4 text-muted-foreground"
                    >
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2" />
                    </svg>
                </CardHeader>
                <CardContent>
                    <div class="text-2xl font-bold">{{ formatMoney(stats.balance) }}</div>
                </CardContent>
            </Card>
        </div>

        <!-- Transactions Table -->
        <div class="rounded-md border flex-1 overflow-hidden">
            <ScrollArea class="h-full">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead class="w-[200px]">Date</TableHead>
                            <TableHead>Merchant</TableHead>
                            <TableHead>Description</TableHead>
                            <TableHead class="text-right">Amount</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <TableRow v-for="t in transactions" :key="t.id">
                            <TableCell class="font-medium">
                                {{ formatDate(t.received_at) }}
                            </TableCell>
                            <TableCell>
                                {{ t.merchant || t.sender }}
                            </TableCell>
                            <TableCell>
                                {{ t.subject }}
                            </TableCell>
                            <TableCell :class="['text-right font-medium', getAmountColor(t)]">
                                {{ getAmountDisplay(t) }}
                            </TableCell>
                        </TableRow>
                        <TableRow v-if="transactions.length === 0">
                            <TableCell colspan="4" class="h-24 text-center">
                                No transactions found.
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </ScrollArea>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { format } from 'date-fns'

// Components
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { ScrollArea } from '@/components/ui/scroll-area'

interface LedgerStats {
    total_income: number
    total_expense: number
    balance: number
}

interface Email {
    id: string
    sender: string
    merchant?: string
    subject: string
    body_preview: string
    received_at: string
    amount?: number
}

const stats = ref<LedgerStats>({
    total_income: 0,
    total_expense: 0,
    balance: 0
})

const transactions = ref<Email[]>([])

const settings = ref({
    currency_symbol: '$',
    number_locale: 'en-US',
    decimals: true
})

onMounted(async () => {
    try {
        const generalSettings: any = await invoke('get_general_settings')
        if (generalSettings) {
            settings.value = {
                currency_symbol: generalSettings.currency_symbol || '$',
                number_locale: generalSettings.number_locale || 'en-US',
                decimals: generalSettings.decimals ?? true
            }
        }

        stats.value = await invoke('get_ledger_stats')
        transactions.value = await invoke('get_transactions')
    } catch (e) {
        console.error("Failed to load ledger data", e)
    }
})



// Function replacement
function formatMoney(val: number) {
     const formattedNum = new Intl.NumberFormat(settings.value.number_locale, {
       minimumFractionDigits: settings.value.decimals ? 2 : 0,
       maximumFractionDigits: settings.value.decimals ? 2 : 0,
    }).format(val);
    
    return `${settings.value.currency_symbol} ${formattedNum}`;
}

function formatDate(dateStr: string) {
    try {
        return format(new Date(dateStr), 'MMM dd, yyyy')
    } catch {
        return dateStr
    }
}

function isCredit(t: Email): boolean {
    const text = (t.subject + ' ' + t.body_preview).toLowerCase()
    // "Cr", "Deposited"
    // "Cr" is tricky as it might be in "Crazy" or "Cry". Let's look for " cr " or end of string?
    // User said "contains 'Cr', 'Deposited'".
    // For safety against partial matches like "Create", I'll check " cr " or just trust user provided keywords strictly.
    // I'll check for "credit", "deposited", "cr." or just "cr" with word boundaries if possible, but JS string includes is simple.
    // I will use some robust checks.
    if (text.includes('deposited') || text.includes('credit')) return true
    if (/\bcr\b/i.test(text)) return true
    return false
}

function isDebit(t: Email): boolean {
    const text = (t.subject + ' ' + t.body_preview).toLowerCase()
    // "Dr", "Spent", "Paid"
    if (text.includes('spent') || text.includes('paid') || text.includes('debit')) return true
    if (/\bdr\b/i.test(text)) return true
    return false
}

function getAmountColor(t: Email): string {
    if (!t.amount) return 'text-muted-foreground'
    if (isCredit(t)) return 'text-green-500'
    if (isDebit(t)) return 'text-red-500'
    return 'text-foreground' // Neutral if neither detected
}

function getAmountDisplay(t: Email): string {
    if (t.amount === undefined || t.amount === null) return '--'
    
    const formatted = formatMoney(t.amount)
    if (isCredit(t)) return `+ ${formatted}`
    if (isDebit(t)) return `- ${formatted}`
    return formatted
}
</script>
