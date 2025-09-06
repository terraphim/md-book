/**
 * Cloudflare Worker for MD-Book
 * Provides API endpoints, redirects, and edge functions for the documentation site
 */

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const { pathname } = url;

    // Handle API routes
    if (pathname.startsWith('/api/')) {
      return handleApi(request, env, pathname);
    }

    // Handle redirects and legacy URLs
    if (pathname.startsWith('/legacy/') || pathname.startsWith('/old/')) {
      return handleRedirects(pathname);
    }

    // Add security headers to all responses
    const response = await env.ASSETS.fetch(request);
    return addSecurityHeaders(response);
  },
};

/**
 * Handle API endpoints
 */
async function handleApi(request, env, pathname) {
  const apiPath = pathname.replace('/api', '');

  switch (apiPath) {
    case '/health':
      return new Response(JSON.stringify({
        status: 'ok',
        timestamp: new Date().toISOString(),
        version: '1.0.0'
      }), {
        headers: { 'Content-Type': 'application/json' }
      });

    case '/search/suggestions':
      // Enhanced search suggestions using the search index
      return handleSearchSuggestions(request, env);

    case '/analytics/event':
      // Track custom analytics events
      if (request.method === 'POST') {
        return handleAnalyticsEvent(request, env);
      }
      return new Response('Method not allowed', { status: 405 });

    case '/feedback':
      // Handle feedback submissions
      if (request.method === 'POST') {
        return handleFeedback(request, env);
      }
      return new Response('Method not allowed', { status: 405 });

    default:
      return new Response('API endpoint not found', { status: 404 });
  }
}

/**
 * Handle search suggestions
 */
async function handleSearchSuggestions(request, env) {
  const url = new URL(request.url);
  const query = url.searchParams.get('q');
  
  if (!query || query.length < 2) {
    return new Response(JSON.stringify({ suggestions: [] }), {
      headers: { 
        'Content-Type': 'application/json',
        'Cache-Control': 'public, max-age=300'
      }
    });
  }

  // This would integrate with your search index
  // For now, return mock suggestions
  const suggestions = [
    'getting started',
    'configuration',
    'deployment',
    'features',
    'api reference'
  ].filter(s => s.toLowerCase().includes(query.toLowerCase()));

  return new Response(JSON.stringify({ 
    query,
    suggestions: suggestions.slice(0, 5)
  }), {
    headers: { 
      'Content-Type': 'application/json',
      'Cache-Control': 'public, max-age=300'
    }
  });
}

/**
 * Handle analytics events
 */
async function handleAnalyticsEvent(request, env) {
  try {
    const data = await request.json();
    
    // Validate event data
    if (!data.event || !data.page) {
      return new Response('Invalid event data', { status: 400 });
    }

    // Store in KV or send to external analytics service
    // For now, just log and return success
    console.log('Analytics event:', data);
    
    return new Response(JSON.stringify({ success: true }), {
      headers: { 'Content-Type': 'application/json' }
    });
  } catch (error) {
    return new Response('Invalid JSON', { status: 400 });
  }
}

/**
 * Handle feedback submissions
 */
async function handleFeedback(request, env) {
  try {
    const data = await request.json();
    
    // Validate feedback data
    if (!data.message || !data.page) {
      return new Response('Message and page are required', { status: 400 });
    }

    // Store feedback in KV or send to external service
    const feedbackId = crypto.randomUUID();
    
    // You could store this in Cloudflare KV:
    // await env.FEEDBACK.put(feedbackId, JSON.stringify({
    //   ...data,
    //   timestamp: new Date().toISOString(),
    //   ip: request.headers.get('CF-Connecting-IP')
    // }));

    console.log('Feedback received:', { id: feedbackId, ...data });
    
    return new Response(JSON.stringify({ 
      success: true,
      id: feedbackId
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  } catch (error) {
    return new Response('Invalid JSON', { status: 400 });
  }
}

/**
 * Handle URL redirects for legacy or moved content
 */
function handleRedirects(pathname) {
  const redirectMap = {
    '/legacy/docs': '/introduction',
    '/old/getting-started': '/getting-started',
    '/docs/api': '/api-reference',
  };

  const newPath = redirectMap[pathname];
  if (newPath) {
    return Response.redirect(newPath, 301);
  }

  return new Response('Not found', { status: 404 });
}

/**
 * Add security headers to responses
 */
function addSecurityHeaders(response) {
  const newHeaders = new Headers(response.headers);
  
  // Security headers
  newHeaders.set('X-Content-Type-Options', 'nosniff');
  newHeaders.set('X-Frame-Options', 'DENY');
  newHeaders.set('X-XSS-Protection', '1; mode=block');
  newHeaders.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  newHeaders.set('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
  
  // Content Security Policy for documentation sites
  newHeaders.set('Content-Security-Policy', 
    "default-src 'self'; " +
    "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdnjs.cloudflare.com; " +
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; " +
    "font-src 'self' https://fonts.gstatic.com; " +
    "img-src 'self' data: https:; " +
    "connect-src 'self';"
  );

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: newHeaders,
  });
}