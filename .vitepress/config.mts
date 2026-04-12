import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Perdex",
    description: "A decentralized comic platform",
    srcDir: 'docs',
    markdown: {
        theme: {
            light: 'github-light-default',
            dark: 'github-dark-dimmed'
        }
    },
    themeConfig: {
        nav: [
            { text: 'Home', link: '/' },
            { text: 'ActivityPub', link: '/activitypub' },
        ],
        sidebar: [
            {
                text: 'Documentation',
                items: [
                    { text: 'ActivityPub', link: '/activitypub' },
                ]
            }
        ],

        socialLinks: [
            { icon: 'github', link: 'https://github.com/perdex-org/documentation' }
        ]
    }
})