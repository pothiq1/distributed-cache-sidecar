<template>
  <div class="cache-search">
    <h2>Cache Search</h2>
    <form @submit.prevent="searchCache">
      <div class="form-group">
        <label for="key">Cache Key</label>
        <input type="text" class="form-control" id="key" v-model="searchKey" placeholder="Enter cache key">
      </div>
      <button type="submit" class="btn btn-primary">Search</button>
    </form>
    <div v-if="searchResult !== null" class="search-result">
      <h3>Search Result</h3>
      <p v-if="searchResult.found"><strong>Value:</strong> {{ searchResult.value }}</p>
      <p v-else>Key not found in cache.</p>
    </div>
  </div>
</template>

<script>
export default {
  name: 'CacheSearch',
  data() {
    return {
      searchKey: '',
      searchResult: null,
    };
  },
  methods: {
    searchCache() {
      fetch(`/search_cache?key=${encodeURIComponent(this.searchKey)}`)
        .then(response => response.json())
        .then(data => {
          this.searchResult = data;
        });
    },
  },
};
</script>

<style scoped>
.cache-search {
  margin-top: 20px;
}
.search-result {
  margin-top: 20px;
}
</style>
