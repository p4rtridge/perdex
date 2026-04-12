import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "MagtivityPub",
    description: "A decentralized comic networking protocol customized from ActivityPub",
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
            { icon: 'github', link: 'https://github.com/magtivitypub/magtivitypub' }
        ]
    }
})