import { createRouter, createWebHistory } from 'vue-router'
import KanbanView from '../views/KanbanView.vue'
import Inbox from '../views/Inbox.vue'
import ChatView from '../views/ChatView.vue'
import FeedView from '../views/FeedView.vue'
import LedgerView from '../views/LedgerView.vue'
import PulseView from '../views/PulseView.vue'
import Settings from '../views/Settings.vue'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            redirect: '/kanban'
        },
        {
            path: '/inbox',
            name: 'inbox',
            component: Inbox
        },
        {
            path: '/kanban',
            name: 'kanban',
            component: KanbanView
        },
        {
            path: '/chat',
            name: 'chat',
            component: ChatView
        },
        {
            path: '/feed',
            name: 'feed',
            component: FeedView
        },
        {
            path: '/ledger',
            name: 'ledger',
            component: LedgerView
        },
        {
            path: '/pulse',
            name: 'pulse',
            component: PulseView
        },
        {
            path: '/settings',
            name: 'settings',
            component: Settings
        }
    ]
})

export default router
