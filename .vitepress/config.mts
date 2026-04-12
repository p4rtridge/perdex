import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "MagtivityPub",
    description: "A decentralized comic networking protocol customized from ActivityPub",
    srcDir: 'docs',
    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        nav: [
            { text: 'Home', link: '/' },
            { text: 'ActivityPub', link: '/activitypub' },
            { text: 'Guidelines', link: '/guidelines' },
            { text: 'REST API', link: '/rest-api' },
        ],
        sidebar: [
            {
                text: 'Protocol Specification',
                items: [
                    { text: 'ActivityPub', link: '/activitypub' },
                    { text: 'Guidelines', link: '/guidelines' },
                    { text: 'REST API', link: '/rest-api' },
                ]
            }
        ],

        socialLinks: [
            { icon: 'github', link: 'https://github.com/magtivitypub/magtivitypub' }
        ]
    }
})