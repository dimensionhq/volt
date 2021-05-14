$(document).ready(() => {
  $('#heading')
    .typeWrite({
      speed: 70,
      repeat: false,
      cursor: false,
      color: 'orange',
      interval: 1000,
    })
    .then((res) => {
      console.log(res)
      $('#subheading').css('display', 'flex')
      $('#subheading').typeWrite({
        speed: 70,
        repeat: false,
        cursor: true,
        color: 'yellowgreen',
        interval: 1000,
      })
    })
})
