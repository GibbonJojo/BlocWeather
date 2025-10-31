<template>
  <div style="max-width:9000px;margin:28px auto;font-family:system-ui,Segoe UI,Roboto,Arial">
    <h1>Hohenfels</h1>
    <div style="width: 1000px" id="chart">
      <Chart />
    </div>
  </div>
</template>

<script>
import Chart from './components/Chart.vue'

export default {
  name: 'chart',
  components: {
    Chart
  },
}

</script>

<style>
/* no canvas styles needed for ApexCharts component */
</style>