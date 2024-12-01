<template>
  <div class="configuration">
    <h2>Configuration</h2>
    <form @submit.prevent="updateConfig">
      <div class="form-group">
        <label for="maxMemory">Max Memory (bytes)</label>
        <input type="number" class="form-control" id="maxMemory" v-model.number="config.max_memory">
      </div>
      <div class="form-group">
        <label for="defaultTTL">Default TTL (seconds)</label>
        <input type="number" class="form-control" id="defaultTTL" v-model.number="config.default_ttl">
      </div>
      <!-- Add more configuration fields as needed -->
      <button type="submit" class="btn btn-success">Update Configuration</button>
    </form>
  </div>
</template>

<script>
export default {
  name: 'Configuration',
  data() {
    return {
      config: {
        max_memory: 0,
        default_ttl: 0,
      },
    };
  },
  mounted() {
    this.fetchConfig();
  },
  methods: {
    fetchConfig() {
      fetch('/config')
        .then(response => response.json())
        .then(data => {
          this.config = data;
        });
    },
    updateConfig() {
      fetch('/update_config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(this.config),
      })
        .then(response => response.json())
        .then(data => {
          if (data.status === 'success') {
            alert('Configuration updated successfully.');
          } else {
            alert('Failed to update configuration.');
          }
        });
    },
  },
};
</script>

<style scoped>
.configuration {
  margin-top: 20px;
}
</style>
