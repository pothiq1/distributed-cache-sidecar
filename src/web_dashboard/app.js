// app.js

Vue.component('header-component', HeaderComponent);
Vue.component('footer-component', FooterComponent);
Vue.component('cache-stats', CacheStats);
Vue.component('cache-search', CacheSearch);
Vue.component('transaction-manager', TransactionManager);
Vue.component('configuration', Configuration);

new Vue({
  el: '#app',
  data: {
    currentView: 'stats',
  },
  methods: {
    navigate(view) {
      this.currentView = view;
    },
  },
});
