export interface PricingPlan {
  id: string;
  name: string;
  tagline: string;
  price: string;
  priceNote: string;
  highlighted?: boolean;
  ctaLabel: string;
  ctaHref: string;
  features: string[];
}

/** Draft commercial tiers — marketing only; no billing is enforced yet. */
export const pricingPlans: PricingPlan[] = [
  {
    id: 'self-hosted',
    name: 'Self-hosted',
    tagline: 'Full product on your infrastructure.',
    price: '$0',
    priceNote: 'Free forever',
    ctaLabel: 'Get started free',
    ctaHref: '/register?plan=self-hosted',
    features: [
      'Unlimited sites you host',
      'Unlimited events on your servers',
      'Docker Compose deployment',
      'Cookieless tracker & first-party proxy',
      'Real-time Spy, goals, and CSV export',
      'You manage ops & backups'
    ]
  },
  {
    id: 'starter',
    name: 'Starter',
    tagline: 'Hosted analytics without the ops burden.',
    price: '$9',
    priceNote: 'per month',
    ctaLabel: 'Create account',
    ctaHref: '/register?plan=starter',
    features: [
      'Up to 3 sites',
      '100k page views / month',
      '12-month data retention',
      'Cookieless by default',
      'Real-time Spy & goals',
      'Email support'
    ]
  },
  {
    id: 'pro',
    name: 'Pro',
    tagline: 'For growing product and content teams.',
    price: '$29',
    priceNote: 'per month',
    highlighted: true,
    ctaLabel: 'Create account',
    ctaHref: '/register?plan=pro',
    features: [
      'Up to 15 sites',
      '1M page views / month',
      '24-month data retention',
      'Everything in Starter',
      'Priority support',
      'CSV export & multi-site rollup'
    ]
  }
];

export const pricingComparison = [
  { feature: 'Cookieless tracking', selfHosted: true, starter: true, pro: true },
  { feature: 'Real-time Spy (SSE)', selfHosted: true, starter: true, pro: true },
  { feature: 'Pages, referrers, countries, devices, campaigns', selfHosted: true, starter: true, pro: true },
  { feature: 'Goals & custom events', selfHosted: true, starter: true, pro: true },
  { feature: 'First-party anti-adblock proxy', selfHosted: true, starter: true, pro: true },
  { feature: 'CSV export', selfHosted: true, starter: true, pro: true },
  { feature: 'You control the servers', selfHosted: true, starter: false, pro: false },
  { feature: 'Managed hosting', selfHosted: false, starter: true, pro: true },
  { feature: 'Priority support', selfHosted: false, starter: false, pro: true }
] as const;

export const pricingFaqs = [
  {
    question: 'Is billing live today?',
    answer:
      'Pricing describes our planned hosted plans. Create an account to get started — checkout is not required yet. Self-hosted remains free forever on your own infrastructure.'
  },
  {
    question: 'What is included when I self-host?',
    answer:
      'The complete Slimlytics product: multi-site analytics, cookieless tracker, real-time Spy, reports, goals, CSV export, and first-party proxy setup. You run Docker Compose and own the data.'
  },
  {
    question: 'Do you use cookies or sell visitor data?',
    answer:
      'No. Tracking is cookieless by default, visitor IDs are site-scoped, and we never build cross-site advertising profiles or sell visitor data.'
  },
  {
    question: 'Can I move from hosted to self-hosted later?',
    answer:
      'Yes. Export your data via CSV/API and deploy the open stack on your own servers whenever you prefer full control.'
  }
];
