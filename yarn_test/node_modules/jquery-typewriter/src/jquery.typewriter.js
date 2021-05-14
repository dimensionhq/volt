;(function ($) {
  const timer = (ms) => new Promise((res) => setTimeout(res, ms))

  $.fn.typeWrite = function (options) {
    var settings = $.extend(
      {
        speed: 50,
        repeat: false,
        cursor: true,
        color: 'black',
        interval: 1000,
      },
      options
    )

    let speed = 100 - settings.speed

    return new Promise((res, rej) => {
      this.each(async function () {
        var inputText = $(this).text()
        letters = inputText.split('')
        $(this).text('')
        $(this).css('color', settings.color)
        $(this).css('margin', '0px')
        var text = letters[0]
        while (settings.repeat == true) {
          for (let i = 0; i < letters.length; i++) {
            if (settings.cursor == true) {
              if (letters[i + 1] !== undefined) {
                $(this).text(text)
                $('#cursor').remove()
                $(this).append("<span id='cursor'>︳</span>")
                $('#cursor').css('animation', 'blink 1s infinite')
                text = text + letters[i + 1]
                await timer(speed)
              } else {
                $(this).text(text)
                $('#cursor').remove()
                $(this).append("<span id='cursor'>︳</span>")
                text = text + letters[i]
                await timer(settings.interval)
              }
            } else {
              if (letters[i + 1] !== undefined) {
                $(this).text(text)
                text = text + letters[i + 1]
                await timer(speed)
              } else {
                $(this).text(text)
                text = text + letters[i]
                await timer(settings.interval)
              }
            }
          }
          console.log('Clearing text')
          $(this).text('')
          text = letters[0]
        }
        for (let i = 0; i < letters.length; i++) {
          if (settings.cursor == true) {
            if (letters[i + 1] !== undefined) {
              $(this).text(text)
              $('#cursor').remove()
              $(this).append("<span id='cursor'>︳</span>")
              $('#cursor').css('animation', 'blink 1s infinite')
              text = text + letters[i + 1]
              await timer(speed)
            } else {
              $(this).text(text)
              $('#cursor').remove()
              $(this).append("<span id='cursor'>︳</span>")
              text = text + letters[i]
              await timer(speed)
            }
          } else {
            if (letters[i + 1] !== undefined) {
              $(this).text(text)
              text = text + letters[i + 1]
              await timer(speed)
            } else {
              $(this).text(text)
              text = text + letters[i]
              await timer(speed)
            }
          }
        }
        res('Done')
      })
    })
  }
})(jQuery)
