import fastify from 'fastify'
import fastifyStatic from 'fastify-static'
import log4js from 'log4js'
import path from 'path'
import puppeteer from "puppeteer"

const __dirname = path.resolve()

// Init logger
const logger = log4js.getLogger()
logger.level = "debug"

// Require the framework and instantiate it
const server = fastify()

// Declare routes
server.register(fastifyStatic, {
  root: path.join(__dirname, 'public'),
  prefix: '/public/', // optional: default '/'
})

server.post('/api/html/render', async function (request, res) {
  try {
    logger.info("Handling html render request")
    logger.debug(request.query)

    const browser = await puppeteer.launch({
      headless: true,
      args: [
        "--no-sandbox",
        "--disable-gpu",
      ]
    });
    const page = await browser.newPage();

    await page.setContent(request.body);

    const content = await page.$(request.query?.element ?? "table");
    const imageBuffer = await content.screenshot({ omitBackground: true });

    await page.close();
    await browser.close();

    res.type('image/png')
    res.send(imageBuffer)
  }
  catch (err) {
    logger.error(errr)
    throw err
  }
})

// Run the server!
server.listen(3000, '0.0.0.0', function (err, address) {
  if (err) {
    logger.error(err)
    process.exit(1)
  }
  logger.info(`server listening on ${address}`)
})
