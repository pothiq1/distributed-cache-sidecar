<template>
  <div class="cache-stats">
    <h2>Cache Statistics</h2>
    <div class="row">
      <div class="col-md-6">
        <table class="table table-bordered">
          <tr><th>Metric</th><th>Value</th></tr>
          <tr><td>Cache Hits</td><td>{{ stats.cache_hits }}</td></tr>
          <tr><td>Cache Misses</td><td>{{ stats.cache_misses }}</td></tr>
          <tr><td>Memory Usage</td><td>{{ formatBytes(stats.memory_usage) }}</td></tr>
          <tr><td>Number of Entries</td><td>{{ stats.entry_count }}</td></tr>
          <tr><td>Active Containers</td><td>{{ activeContainers.length }}</td></tr>
        </table>
      </div>
      <div class="col-md-6">
        <h3>Memory Usage per Container</h3>
        <table class="table table-bordered">
          <tr><th>Node</th><th>Main Cache</th><th>Replicas</th></tr>
          <tr v-for="node in memoryUsage" :key="node.node">
            <td>{{ node.node }}</td>
            <td>{{ formatBytes(node.main_cache) }}</td>
            <td>{{ formatBytes(node.replicas) }}</td>
          </tr>
        </table>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'CacheStats',
  data() {
    return {
      stats: {},
      activeContainers: [],
      memoryUsage: [],
    };
  },
  mounted() {
    this.fetchStats();
    this.fetchNodes();
    this.fetchMemoryUsage();
    setInterval(() => {
      this.fetchStats();
      this.fetchNodes();
      this.fetchMemoryUsage();
    }, 5000);
  },
  methods: {
    fetchStats() {
      fetch('/stats')
        .then(response => response.json())
        .then(data => {
          this.stats = data;
        });
    },
    fetchNodes() {
      fetch('/nodes')
        .then(response => response.json())
        .then(data => {
          this.activeContainers = data;
        });
    },
    fetchMemoryUsage() {
      fetch('/memory_usage')
        .then(response => response.json())
        .then(data => {
          this.memoryUsage = data;
        });
    },
    formatBytes(bytes) {
      if (bytes === 0) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
  },
};
</script>

<style scoped>
.cache-stats h2 {
  margin-top: 0;
}
</style>
