<template>
  <nav class="navbar navbar-inverse navbar-fixed-top">
    <div class="container-fluid">
      <div class="navbar-header">
        <a class="navbar-brand" href="#">Distributed Cache Dashboard</a>
      </div>
      <ul class="nav navbar-nav">
        <li><a href="#" @click.prevent="$emit('navigate', 'stats')">Statistics</a></li>
        <li><a href="#" @click.prevent="$emit('navigate', 'search')">Cache Search</a></li>
        <li><a href="#" @click.prevent="$emit('navigate', 'config')">Configuration</a></li>
        <!-- Add more navigation links as needed -->
      </ul>
    </div>
  </nav>
</template>

<script>
export default {
  name: 'HeaderComponent',
};
</script>

<style scoped>
.navbar-brand {
  font-size: 24px;
}
</style>
