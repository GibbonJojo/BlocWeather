<template>
    <div class="timeseries-chart">
        <apexcharts id="chart" ref="chart" :options="chartOptions" :series="series"></apexcharts>
    </div>
</template>

<script>
import VueApexCharts from 'vue3-apexcharts'

export default {
    name: 'Chart',
    components: {
        apexcharts: VueApexCharts,
    },
    data: function() {
        return {
                loading: false,
                chart: {
                    height: 450,
                    width: "100%",
                    type: 'line',
                    toolbar: {
                        show: false,
                    },
                },
                annotations: {
                    xaxis: [
                        {
                            x: new Date().getTime(),
                            borderColor: '#00E396',
                            label: {
                                borderColor: '#00E396',
                                orientation: 'horizontal',
                                text: 'Now'
                            }
                        }
                    ]
                },
                // set series colors: first = red (line), second = blue (bars)
                colors: ["#ff3b30", "#007aff", "#00CFCF"],
                stroke: {
                    curve: 'smooth',
                    // for mixed charts, allow stroke widths per series (line thicker, bars default)
                    width: [3, 0, 3],
                    dashArray: [0, 0, 2],
                },
                dataLabels: {
                    enabled: false,
                    formatter: function(value, { seriesIndex, dataPointIndex, w }) {
                        return dataPointIndex % 5 == 0 ? value : ""
                    },
                },
                xaxis: {
                    type: 'datetime',
                    labels: {
                        // formatter: function (value, timestamp) {
                        //     const ts = (typeof timestamp === 'number')
                        //         ? timestamp
                        //         : (typeof value === 'number' ? value : Date.parse(value));
                        //     return new Date(ts).toLocaleString();
                        // },
                        format: "ddd"
                    }
                },
                yaxis: [
                    {
                        title: {
                            text: "Temperature (°C)",
                        },
                        min: function (min) {
                            return min < 0 ? min : 0;
                        },
                        max: function (max) {
                            return max > 20 ? max : 20;
                        },
                        labels: {
                            formatter: function (value) {
                                return value.toFixed(0) + "°C";
                            }
                        }
                    },
                    {
                        opposite: true,
                        title: {
                            text: "Precipitation (mm)",
                        },
                        min: 0,
                        max: function (max) {
                            return max > 5 ? max : 5;
                        },
                        labels: {
                            formatter: function (value) {
                                return value.toFixed(1) + " mm";
                            }
                        }
                    },
                    {
                        show: false,
                        min: 0,
                        max: 100,
                        labels: {
                            formatter: function (value) {
                                return value.toFixed(0) + "%";
                            }
                        }
                    }
                ],
                series: [{},{}],
                noData: {
                    text: 'Loading...'
                },
                tooltip: {
                    x: {
                        formatter: function (value, timestamp) {
                            const ts = (typeof timestamp === 'number')
                                ? timestamp
                                : (typeof value === 'number' ? value : Date.parse(value));
                            return new Date(ts).toLocaleString();
                        },
                    }
                }
        }
    },
    computed: {
        chartOptions () {
            // show more x-axis ticks based on the number of data points (cap at 20)
            const points = (this.series && this.series[0] && this.series[0].data) ? this.series[0].data.length : 0;
            const tickAmount = points ? Math.min(20, points) : 10;

            const xaxisOptions = Object.assign({}, this.xaxis, {
                tickAmount,
                labels: Object.assign({}, this.xaxis.labels, {
                    // rotate labels so more can fit without overlapping
                    rotate: -30
                })
            });

            return {
                chart: this.chart,
                noData: this.noData,
                annotations: this.annotations,
                stroke: this.stroke,
                xaxis: xaxisOptions,
                yaxis: this.yaxis,
                tooltip: this.tooltip,
                colors: this.colors,
            }
        }
    },
    methods: {
        async loadData () {
            this.loading = true
            try {
                const temp_res = await fetch('/api/timeseries/hohenfels/temperature')
                if (!temp_res.ok) throw new Error('Network response was not ok')
                const temp = await temp_res.json()
                
                const precip_res = await fetch('/api/timeseries/hohenfels/precipitation')
                if (!precip_res.ok) throw new Error('Network response was not ok')
                const precip = await precip_res.json()

                const humid_res = await fetch('/api/timeseries/hohenfels/humidity')
                if (!humid_res.ok) throw new Error('Network response was not ok')
                const humid = await humid_res.json()

                this.series = [{
                    name: "Temperature",
                    data: temp
                },
                {
                    name: "Precipitation",
                    type: "bar",
                    data: precip
                },
                {
                    name: "Humidity",
                    data: humid
                }]
            } catch (err) {
                console.error('Failed to load timeseries', err)
                this.series = []
            } finally {
                this.loading = false
            }
        }
    },
    mounted () {
        this.loadData()
    }
}

</script>
