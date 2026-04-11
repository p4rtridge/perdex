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
      { text: 'Architecture', link: '/architecture' },
      { text: 'REST API', link: '/rest-api' },
      { text: 'ActivityPub', link: '/activitypub' }
    ],
    sidebar: [
      {
        text: 'Protocol Specification',
        items: [
          { text: 'Architecture', link: '/architecture' },
          { text: 'REST API', link: '/rest-api' },
          { text: 'ActivityPub', link: '/activitypub' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/magtivitypub/magtivitypub' }
    ]
  }
})
