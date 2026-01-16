import { createRouter, createWebHistory } from 'vue-router'
import Kanban from '../views/Kanban.vue'
import Inbox from '../views/Inbox.vue'
import ChatView from '../views/Chat.vue'
import Feed from '../views/Feed.vue'
import Ledger from '../views/Ledger.vue'
import Pulse from '../views/Pulse.vue'
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
            component: Kanban
        },
        {
            path: '/chat',
            name: 'chat',
            component: ChatView
        },
        {
            path: '/feed',
            name: 'feed',
            component: Feed
        },
        {
            path: '/ledger',
            name: 'ledger',
            component: Ledger
        },
        {
            path: '/pulse',
            name: 'pulse',
            component: Pulse
        },

        {
            path: '/settings',
            name: 'settings',
            component: Settings
        }
    ]
})

export default router
